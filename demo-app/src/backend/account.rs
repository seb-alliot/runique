//! An account deleted by its owner (GDPR right to erasure): the current
//! password confirms it, its contributions go with it (`ON DELETE CASCADE`),
//! and every session it holds is closed.
use crate::backend::auth::get_profile_user;
use crate::backend::lang::{localize_errors, msg};
use crate::formulaire::DeleteAccountForm;
use runique::auth::Rang;
use runique::prelude::runique_users::{Column as UserColumn, Entity as UserEntity};
use runique::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// The profile page, with the deletion form as given: its errors show under
/// its field, in the visitor's language. `open` unfolds the deletion block.
pub async fn render_profile(
    request: &mut Request,
    delete_form: DeleteAccountForm,
    open: bool,
) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let user_id = request.user.as_ref().map(|user| user.id);
    let username = request.user.as_ref().map(|user| user.username.clone());
    let (user_opt, user_by_username) =
        get_profile_user(user_id, username.as_deref(), &request.engine.db).await;
    context_update!(request => {
        "title"        => "My profile",
        "delete_form"  => &delete_form,
        "delete_open"  => &open,
        "username"     => &username,
        "profile_user" => &user_opt,
        "profile_user_by_username" => &user_by_username,
        "connected"    => &true,
    });
    request.render("profile/profile.html")
}

pub async fn handle_delete_account(
    request: &mut Request,
    form: DeleteAccountForm,
) -> AppResult<Response> {
    let Some((user_id, username)) = request
        .user
        .as_ref()
        .map(|user| (user.id, user.username.clone()))
    else {
        return Ok(Redirect::to("/login").into_response());
    };
    // `request.form()` refuses a missing or forged CSRF token, and a GET is
    // never validated: either way nothing is read, nothing deleted.
    let mut form = match ValidationForm::try_form(form, request).await {
        Ok(validated) => validated.into_form(),
        Err(mut form) => {
            localize_errors(form.forms_mut(), &request.headers);
            let open = form.is_submitted();
            return render_profile(request, form, open).await;
        }
    };
    let password = form.cleaned::<String>("password").unwrap_or_default();

    let db = request.engine.db.clone();
    // Runique's sign-in check: same time whatever the outcome, an inactive
    // account refused like a wrong password.
    // The account found by name must be the session's own: names are unique
    // today, this holds if an account may one day change its name.
    let account = authenticate_user(&db, &username, &password)
        .await
        .filter(|account| account.id == user_id);
    let Some(account) = account else {
        let error = msg(
            &request.headers,
            "Wrong password: your account was not deleted.",
            "Mot de passe incorrect : votre compte n'a pas été supprimé.",
        );
        if let Some(field) = form.forms_mut().fields.get_mut("password") {
            field.set_error(error.to_string());
        }
        return render_profile(request, form, true).await;
    };
    // A ranked account goes through the rank management: the site must keep a
    // dirigeant while admins remain.
    if !matches!(Rang::from(&account), Rang::Lambda) {
        let error = msg(
            &request.headers,
            "A team account cannot delete itself: ask a dirigeant.",
            "Un compte de l'équipe ne peut pas se supprimer lui-même : adressez-vous à un dirigeant.",
        );
        form.forms_mut().errors.push(error.to_string());
        return render_profile(request, form, true).await;
    }

    // The rank is checked again in the statement itself: a promotion between
    // the read above and this write deletes nothing.
    let deleted = UserEntity::delete_many()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::IsSuperuser.eq(false))
        .filter(UserColumn::IsDirigeant.eq(false))
        .filter(UserColumn::IsAdmin.eq(false))
        .exec(&db)
        .await;
    match deleted {
        Ok(result) if result.rows_affected == 1 => {}
        Ok(_) => {
            let error = msg(
                &request.headers,
                "Your account could not be deleted, try again later.",
                "Votre compte n'a pas pu être supprimé, réessayez plus tard.",
            );
            form.forms_mut().errors.push(error.to_string());
            return render_profile(request, form, true).await;
        }
        Err(e) => {
            tracing::error!(error = %e, "account deletion: delete refused by the database");
            form.forms_mut().database_error(&e);
            return render_profile(request, form, true).await;
        }
    }

    // The account is gone: its sessions on other devices end too, and this one.
    if let Err(e) = request.engine.close_user_sessions(user_id).await {
        tracing::warn!(error = %e, "account deletion: sessions not all closed");
    }
    logout(&request.session, None).await?;
    success!(request.notices => msg(&request.headers, "Your account has been deleted.", "Votre compte a été supprimé."));
    Ok(Redirect::to("/").into_response())
}
