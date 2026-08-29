//! Génère des `Pk` de test distincts et déterministes, portables entre les
//! types de PK (`i32` par défaut, `i64` sous `big-pk`, `Uuid` sous `pk-uuid`).
//! Deux appels avec des `n` différents donnent toujours des `Pk` différents.

use runique::utils::config::Pk;

#[cfg(feature = "pk-uuid")]
pub fn pk(n: u32) -> Pk {
    uuid::Uuid::from_u128(u128::from(n))
}

#[cfg(not(feature = "pk-uuid"))]
pub fn pk(n: u32) -> Pk {
    n as Pk
}

/// Littéral SQL prêt à spliquer dans un `INSERT ... VALUES (...)` écrit à la main,
/// pour la valeur `pk(n)`. Un `Uuid` est stocké par sqlx/SQLite en BLOB brut (16
/// octets), jamais en texte hyphéné — un literal `'xxxx-xxxx-...'` inséré tel quel
/// via `execute_unprepared` ne serait pas relisible par le décodeur typé de SeaORM.
#[cfg(feature = "pk-uuid")]
pub fn pk_sql_literal(n: u32) -> String {
    let bytes = pk(n).into_bytes();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("X'{hex}'")
}

#[cfg(not(feature = "pk-uuid"))]
pub fn pk_sql_literal(n: u32) -> String {
    pk(n).to_string()
}
