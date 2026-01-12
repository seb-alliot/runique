use crate::formulaire::field::RuniqueField;
use std::net::IpAddr;

pub struct IPAddressField;

impl IPAddressField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IPAddressField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for IPAddressField {
    type Output = IpAddr;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .parse::<IpAddr>()
            .map_err(|_| "Adresse IP invalide.".to_string())
    }

    fn template_name(&self) -> &str {
        "ipaddress"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "type": "ip" })
    }
}
