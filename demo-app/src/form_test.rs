use runique::prelude::*;

use runique::forms::field::RuniqueForm;
use runique::forms::fields::choice::ChoiceOption;
use runique::forms::fields::*;
use runique::forms::manager::Forms;
use serde::Serialize;

// ─── Déclaration de la forme ─────────────────────────────────────────────
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct TestAllFieldsForm {
    pub form: Forms,
}

// ─── Implémentation du trait ─────────────────────────────────────────────

#[async_trait::async_trait]
impl RuniqueForm for TestAllFieldsForm {
    fn register_fields(form: &mut Forms) {
        // ── Text ──
        form.field(&TextField::text("f_text").label("Text"));
        form.field(&TextField::email("f_email").label("Email"));
        form.field(&TextField::url("f_url").label("URL"));
        form.field(
            &TextField::password("f_password")
                .label("Mot de passe")
                .min_length(8, "Min 8 caractères"),
        );
        form.field(&TextField::textarea("f_textarea").label("Textarea"));
        form.field(&TextField::richtext("f_richtext").label("Rich Text"));
        form.field(
            &TextField::text("f_readonly")
                .label("Readonly")
                .readonly("Ce champ est en lecture seule"),
        );
        form.field(
            &TextField::text("f_disabled")
                .label("Disabled")
                .disabled("Ce champ est désactivé"),
        );

        // ── Numeric ──
        form.field(
            &NumericField::integer("f_integer")
                .label("Integer")
                .min(0.0, "Min 0")
                .max(1000.0, "Max 1000"),
        );
        form.field(
            &NumericField::float("f_float")
                .label("Float")
                .min(0.0, "")
                .max(99.99, ""),
        );
        form.field(
            &NumericField::decimal("f_decimal")
                .label("Decimal")
                .digits(2, 4),
        );
        form.field(&NumericField::percent("f_percent").label("Percent"));
        form.field(
            &NumericField::range("f_range", 0.0, 100.0, 50.0)
                .label("Range")
                .step(5.0),
        );

        // ── Boolean ──
        form.field(&BooleanField::new("f_checkbox").label("Je teste le checkbox"));
        form.field(&BooleanField::radio("f_radio_single").label("Je teste le radio single"));

        // ── Choice ──
        let choices = vec![
            ChoiceOption::new("opt_a", "Option A"),
            ChoiceOption::new("opt_b", "Option B"),
            ChoiceOption::new("opt_c", "Option C"),
        ];

        form.add(
            ChoiceField::new("f_select")
                .label("Select")
                .choices(choices.clone()),
        );
        form.add(
            ChoiceField::new("f_select_multiple")
                .label("Select multiple")
                .multiple()
                .choices(choices.clone()),
        );
        form.add(
            RadioField::new("f_radio_group")
                .label("Radio group")
                .choices(choices.clone()),
        );
        form.add(
            CheckboxField::new("f_checkbox_group")
                .label("Checkbox group")
                .choices(choices.clone()),
        );

        // ── Datetime ──
        form.add(DateField::new("f_date").label("Date"));
        form.add(TimeField::new("f_time").label("Heure"));
        form.add(DateTimeField::new("f_datetime").label("Date + Heure"));
        form.add(DurationField::new("f_duration").label("Durée (secondes)"));

        // ── File ──
        form.field(
            &FileField::image("f_file_image")
                .label("Image")
                .max_size_mb(5),
        );
        form.field(
            &FileField::document("f_file_document")
                .label("Document")
                .max_size_mb(10),
        );
        form.field(
            &FileField::any("f_file_any")
                .label("Fichier quelconque")
                .max_files(3),
        );

        // ── Special ──
        form.add(
            ColorField::new("f_color")
                .label("Couleur")
                .default_color("#3498db"),
        );
        form.add(
            SlugField::new("f_slug")
                .label("Slug")
                .placeholder("mon-slug-url"),
        );
        form.add(
            UUIDField::new("f_uuid")
                .label("UUID")
                .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
        );
        form.add(
            JSONField::new("f_json")
                .label("JSON")
                .placeholder(r#"{"clé": "valeur"}"#),
        );
        form.add(
            IPAddressField::new("f_ip")
                .label("Adresse IP")
                .placeholder("192.168.1.1"),
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
