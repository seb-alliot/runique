use runique::prelude::*;
use crate::models::test;
use crate::sea_orm::DbErr;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;




// --- FORMULAIRE USERNAME ---
#[derive(Deserialize)]
pub struct UsernameForm {
    #[serde(skip_deserializing)]
    pub form: Forms,
}

impl Serialize for UsernameForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.register_field("username", "Nom d'utilisateur", &CharField::new());
    }

    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
        form.require("username", &CharField::new(), raw_data);
    }

    fn from_form(form: Forms) -> Self { Self { form } }
    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
}

// --- FORMULAIRE REGISTER ---

#[derive(Deserialize)]
pub struct RegisterForm {
    #[serde(skip_deserializing)]
    pub form: Forms,
}

impl Serialize for RegisterForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}


impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.register_field("phone","numéro", &PhoneField::new().required().with_country(CountryId::FR));
        form.register_field("color", "Couleur", &ColorField::new());
        form.register_field("uuid", "UUID", &UUIDField::new());
        form.register_field("description", "Description", &TextareaField::new());
        form.register_field(
            "postal_code",
            "Code Postal",
            &PostalCodeField::new(Country::France),
        );
        // NUMÉRIQUE (Utilisation de PercentageField)
        form.register_field("price", "Prix", &DecimalField::new(10, 2));
        form.register_field("rating", "Note", &RangeField::new(-5, 0, 5));
        form.register_field("quantity", "Quantité", &PositiveIntegerField::new());
        form.register_field("discount", "Remise (%)", &PercentageField::new());
        form.register_field("amount", "Montant", &CurrencyField::new("EUR"));

        // TEMPOREL
        form.register_field("opening_time", "Heure d'ouverture", &TimeField::allow_blank());
        form.register_field("duration", "Durée", &DurationField::allow_blank());

        // FICHIERS
        form.register_field(
            "profile_image",
            "Image de profil",
            &ImageField::new().upload_to("media")
        );
        form.register_field("attachments", "Pièces jointes", &MultipleFileField::new());

        // CHOIX (MultipleChoiceField et RadioSelectField selon tes imports)
        form.register_field(
            "preferences",
            "Préférences",
            &MultipleChoiceField {
                options: to_options(vec![
                    ("email", "Notifications par email"),
                    ("sms", "Notifications par SMS"),
                    ("push", "Notifications push"),
                ]),
            },
        );

        form.register_field(
            "subscription",
            "Abonnement",
            &RadioSelectField {
                options: to_options(vec![
                    ("free", "Gratuit"),
                    ("premium", "Premium"),
                    ("enterprise", "Entreprise"),
                ]),
            },
        );
    }

    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
        form.optional("phone", &PhoneField::new(), raw_data);
        form.optional("color", &ColorField::new(), raw_data);
        form.optional("uuid", &UUIDField::new(), raw_data);
        form.optional("description", &TextareaField::new(), raw_data);
        form.optional("postal_code", &PostalCodeField::new(Country::France), raw_data);

        form.optional("price", &DecimalField::new(10, 2), raw_data);
        form.optional("rating", &RangeField::new(-5, 0, 5), raw_data);
        form.optional("quantity", &PositiveIntegerField::new(), raw_data);
        form.optional("discount", &PercentageField::new(), raw_data);
        form.optional("amount", &CurrencyField::new("EUR"), raw_data);

        form.optional("opening_time", &TimeField::allow_blank(), raw_data);
        form.optional("duration", &DurationField::allow_blank(), raw_data);

        form.optional("profile_image", &ImageField::new().upload_to("media"), raw_data);
        form.optional("attachments", &MultipleFileField::new().upload_to("static/document"), raw_data);

        form.optional(
            "preferences",
            &MultipleChoiceField {
                options: to_options(vec![("email", "Notifications par email"), ("sms", "Notifications par SMS"), ("push", "Notifications push")]),
            },
            raw_data,
        );
        form.optional(
            "subscription",
            &RadioSelectField {
                options: to_options(vec![("free", "Gratuit"), ("premium", "Premium"), ("enterprise", "Entreprise")]),
            },
            raw_data,
        );
    }

    fn from_form(form: Forms) -> Self { Self { form } }
    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
}
