use crate::models::test;
use crate::sea_orm::DbErr;

use runique::formulaire::field::{
    CharField, ColorField, CurrencyField, DecimalField, DurationField, ImageField,
    MultipleChoiceField, MultipleFileField, PercentageField, PhoneField, PositiveIntegerField,
    PostalCodeField, RadioSelectField, RangeField, TextField, TimeField, UUIDField,
};
use runique::prelude::*;
use runique::Country;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

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

    fn from_form(form: Forms) -> Self {
        Self { form }
    }

    fn get_form(&self) -> &Forms {
        &self.form
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

#[derive(Deserialize)]
pub struct TestFieldsForm {
    #[serde(skip_deserializing)]
    pub form: Forms,
}

impl Serialize for TestFieldsForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for TestFieldsForm {
    fn register_fields(form: &mut Forms) {
        // TEXTE AVANCÉ
        form.register_field("phone", "Téléphone", &PhoneField);
        form.register_field("color", "Couleur", &ColorField);
        form.register_field("uuid", "UUID", &UUIDField);
        form.register_field("description", "Description", &TextField::new());
        form.register_field(
            "postal_code",
            "Code Postal",
            &PostalCodeField::new(Country::France),
        );

        // NUMÉRIQUE AVANCÉ
        form.register_field("price", "Prix", &DecimalField::new(10, 2));
        form.register_field("rating", "Note", &RangeField::new(1, 5));
        form.register_field("quantity", "Quantité", &PositiveIntegerField);
        form.register_field("discount", "Remise (%)", &PercentageField);
        form.register_field("amount", "Montant", &CurrencyField::new("EUR"));

        // TEMPOREL AVANCÉ
        form.register_field("opening_time", "Heure d'ouverture", &TimeField);
        form.register_field("duration", "Durée", &DurationField);

        // FICHIERS
        form.register_field("profile_image", "Image de profil", &ImageField::new());
        form.register_field(
            "attachments",
            "Pièces jointes",
            &MultipleFileField::new("*/*"),
        );

        // CHOIX
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
        // Tous les champs en OPTIONAL
        form.optional("phone", &PhoneField, raw_data);
        form.optional("color", &ColorField, raw_data);
        form.optional("uuid", &UUIDField, raw_data);
        form.optional("description", &TextField::new(), raw_data);
        form.optional(
            "postal_code",
            &PostalCodeField::new(Country::France),
            raw_data,
        );
        form.optional("price", &DecimalField::new(10, 2), raw_data);
        form.optional("rating", &RangeField::new(1, 5), raw_data);
        form.optional("quantity", &PositiveIntegerField, raw_data);
        form.optional("discount", &PercentageField, raw_data);
        form.optional("amount", &CurrencyField::new("EUR"), raw_data);
        form.optional("opening_time", &TimeField, raw_data);
        form.optional("duration", &DurationField, raw_data);

        // FICHIERS - Optional
        form.optional("image", &ImageField::new(), raw_data);
        form.optional("attachments", &MultipleFileField::new("*/*"), raw_data);

        // CHOIX - Optional
        form.optional(
            "preferences",
            &MultipleChoiceField {
                options: to_options(vec![("email", "Email"), ("sms", "SMS"), ("push", "Push")]),
            },
            raw_data,
        );
        form.optional(
            "subscription",
            &RadioSelectField {
                options: to_options(vec![
                    ("free", "Gratuit"),
                    ("premium", "Premium"),
                    ("enterprise", "Entreprise"),
                ]),
            },
            raw_data,
        );
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }

    fn get_form(&self) -> &Forms {
        &self.form
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}
impl TestFieldsForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<test::Model, DbErr> {
        let f = &self.form;

        let active_model = test::ActiveModel {
            phone: Set(f.get_value("phone").unwrap_or_default()),
            color: Set(f.get_value("color").unwrap_or_default()),
            uuid: Set(f.get_value("uuid").unwrap_or_default()),
            description: Set(f.get_value("description").unwrap_or_default()),
            postal_code: Set(f.get_value("postal_code").unwrap_or_default()),

            // NUMÉRIQUE AVANCÉ
            price: Set(f.get_value("price").unwrap_or_default()),

            rating: Set(f
                .get_value("rating")
                .and_then(|v: String| v.parse::<i64>().ok())
                .unwrap_or(0i64)),

            quantity: Set(f
                .get_value("quantity")
                .and_then(|v: String| v.parse::<i64>().ok())
                .unwrap_or(0i64)),

            discount: Set(f
                .get_value("discount")
                .and_then(|v: String| v.parse::<f64>().ok())
                .unwrap_or(0.0f64)),

            amount: Set(f.get_value("amount").unwrap_or_default()),

            // TEMPOREL AVANCÉ
            opening_time: Set(f.get_value("opening_time").unwrap_or_default()),

            duration: Set(f
                .get_value("duration")
                .and_then(|v: String| v.parse::<i64>().ok())
                .unwrap_or(0)),

            // FICHIERS
            image: Set(f.get_value("image").unwrap_or_default()),
            attachments: Set(f.get_value("attachments").unwrap_or_default()),

            // CHOIX
            preferences: Set(f.get_value("preferences").unwrap_or_default()),
            subscription: Set(f.get_value("subscription").unwrap_or_default()),
            created_at: Set(chrono::Utc::now().naive_utc()),

            ..Default::default()
        };

        let insert_result = test::Entity::insert(active_model).exec(db).await?;

        let model = test::Entity::find_by_id(insert_result.last_insert_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom("Enregistrement non trouvé après insertion".to_string())
            })?;

        Ok(model)
    }
}
