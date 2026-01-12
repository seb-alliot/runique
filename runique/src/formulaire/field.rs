use serde_json::Value;
use tera::Context;
use tera::Tera;

pub struct SelectOption {
    pub value: String,
    pub label: String,
}

pub trait RuniqueField {
    type Output;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;
    fn template_name(&self) -> &str;

    fn strip(&self) -> bool {
        true
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// La méthode qui rend le template Tera
    fn render(
        &self,
        tera: &Tera,
        name: &str,
        label: &str,
        value: &Value,
        error: Option<&String>,
    ) -> String {
        let mut ctx = Context::new();
        ctx.insert("name", name);
        ctx.insert("label", label);
        ctx.insert("id", &format!("id_{}", name));
        ctx.insert("value", value);

        if let Some(err) = error {
            ctx.insert("error", err);
        }

        if let Value::Object(map) = self.get_context() {
            for (k, v) in map {
                ctx.insert(k, &v);
            }
        }

        tera.render(self.template_name(), &ctx)
            .unwrap_or_else(|e| format!("Erreur Tera ({}): {}", self.template_name(), e))
    }
}
