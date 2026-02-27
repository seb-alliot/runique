// Tests pour error_handler_middleware (structure)
#[test]

use runique::middleware::errors::error::RequestInfoHelper;
use std::collections::HashMap;

#[test]
fn test_request_info_helper_struct() {
    let mut headers = HashMap::new();
    headers.insert("x-test-header".to_string(), "valeur".to_string());
    let helper = RequestInfoHelper {
        method: "GET".to_string(),
        path: "/".to_string(),
        query: Some("a=1".to_string()),
        headers: headers.clone(),
    };
    assert_eq!(helper.method, "GET");
    assert_eq!(helper.path, "/");
    assert_eq!(helper.query, Some("a=1".to_string()));
    assert_eq!(helper.headers.get("x-test-header").unwrap(), "valeur");
}
