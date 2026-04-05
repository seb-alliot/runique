pub mod cour;
pub mod demo;
pub mod doc;
pub mod ia;

use runique::prelude::*;

pub async fn run_seeds(db: &DatabaseConnection) {
    doc::seed_docs(db).await;
    cour::seed_cours(db).await;
    // demo::seed_demo(db).await;
    ia::seed_ia(db).await;
}
