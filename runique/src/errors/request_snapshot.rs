//! What an error page keeps of the request it answers. The error middleware
//! fills it, leaving out the headers and the path segments that hold a secret.
use crate::utils::aliases::StrMap;

/// Transport for request info used in contextual debug
pub struct RequestSnapshot {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: StrMap,
}
