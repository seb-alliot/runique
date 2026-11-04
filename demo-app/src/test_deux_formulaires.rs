use crate::formulaire::{EmailForm, LoginForm};
use runique::prelude::*;

#[derive(Serialize)]
struct Constat {
    formulaire: &'static str,
    resultat: &'static str,
    methode_sure: bool,
    is_submitted: bool,
    erreurs_de_champ: usize,
}

pub async fn deux_formulaires(mut request: Request) -> AppResult<Response> {
    let login: LoginForm = request.form();
    let email: EmailForm = request.form();
    let methode = request.method.to_string();
    let methode_sure = request.method.is_safe();

    let (login, constat_login) = constat(
        ValidationForm::try_form(login, &request, "login").await,
        "login",
        methode_sure,
    );
    let (email, constat_email) = constat(
        ValidationForm::try_form(email, &request, "email").await,
        "email",
        methode_sure,
    );

    context_update!(request => {
        "methode"    => &methode,
        "constats"   => &[constat_login, constat_email],
        "login_form" => &login,
        "email_form" => &email,
    });
    request.render("test_deux_formulaires.html")
}

fn constat<F: RuniqueForm>(
    tried: Result<ValidationForm<F>, F>,
    formulaire: &'static str,
    methode_sure: bool,
) -> (F, Constat) {
    let (form, resultat) = match tried {
        Ok(validated) => (validated.into_form(), "Ok"),
        Err(form) => (form, "Err"),
    };
    let erreurs_de_champ = form
        .forms()
        .fields
        .values()
        .filter(|field| field.error().is_some())
        .count();
    let constat = Constat {
        formulaire,
        resultat,
        methode_sure,
        is_submitted: form.is_submitted(),
        erreurs_de_champ,
    };
    (form, constat)
}

/*  memo perso => version simplifier plus haut, complete en bas
fn constat<F: RuniqueForm>(
    tried: Result<ValidationForm<F>, F>,
    formulaire: &'static str,
    methode_sure: bool,
) -> (F, Constat) {
    // 1. Récupérer le formulaire et noter si try_form a dit Ok ou Err
    let form: F;
    let resultat: &'static str;

    if let Ok(validated) = tried {
        form = validated.into_form();
        resultat = "Ok";
    } else if let Err(refuse) = tried {
        form = refuse;
        resultat = "Err";
    } else {
        unreachable!();
    }

    // 2. Compter les champs en erreur
    let mut erreurs_de_champ = 0;
    for field in form.forms().fields.values() {
        if field.error().is_some() {
            erreurs_de_champ += 1;
        }
    }

    // 3. Construire le constat
    let constat = Constat {
        formulaire: formulaire,
        resultat: resultat,
        methode_sure: methode_sure,
        is_submitted: form.is_submitted(),
        erreurs_de_champ: erreurs_de_champ,
    };
    (form, constat)
} */
