use crate::moteur_engine::engine_struct::RuniqueEngine;
use std::sync::Arc;
use tera::Context;

pub struct TemplateEngine {
    pub engine: Arc<RuniqueEngine>,
    pub context: Context,
}

impl TemplateEngine {
    pub fn new(engine: Arc<RuniqueEngine>, csrf_token: String) -> Self {
        let mut context = Context::new();
        context.insert("static_runique", &engine.config.static_files);
        context.insert("csrf_token", &csrf_token);
        Self { engine, context }
    }
}
