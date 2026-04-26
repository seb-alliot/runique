//! Tests — forms/options/bool_choice.rs
//! Couvre : BoolChoice::new, Default, champs choice et message.

use runique::forms::options::BoolChoice;

#[test]
fn test_bool_choice_new_true_avec_message() {
    let bc = BoolChoice::new(true, Some("Requis".to_string()));
    assert!(bc.choice);
    assert_eq!(bc.message, Some("Requis".to_string()));
}

#[test]
fn test_bool_choice_new_false_sans_message() {
    let bc = BoolChoice::new(false, None);
    assert!(!bc.choice);
    assert_eq!(bc.message, None);
}

#[test]
fn test_bool_choice_default() {
    let bc = BoolChoice::default();
    assert!(!bc.choice);
    assert_eq!(bc.message, None);
}

#[test]
fn test_bool_choice_clone() {
    let bc = BoolChoice::new(true, Some("msg".to_string()));
    let cloned = bc.clone();
    assert_eq!(cloned.choice, bc.choice);
    assert_eq!(cloned.message, bc.message);
}

#[test]
fn test_bool_choice_debug() {
    let bc = BoolChoice::new(true, Some("test".to_string()));
    let s = format!("{:?}", bc);
    assert!(s.contains("BoolChoice"));
}

#[test]
fn test_bool_choice_serialize() {
    let bc = BoolChoice::new(true, Some("msg".to_string()));
    let json = serde_json::to_string(&bc).unwrap();
    assert!(json.contains("\"choice\":true"));
    assert!(json.contains("\"message\":\"msg\""));
}

#[test]
fn test_bool_choice_serialize_none() {
    let bc = BoolChoice::new(false, None);
    let json = serde_json::to_string(&bc).unwrap();
    assert!(json.contains("\"choice\":false"));
    assert!(json.contains("\"message\":null"));
}

#[test]
fn test_bool_choice_deserialize() {
    let json = r#"{"choice":true,"message":"Obligatoire"}"#;
    let bc: BoolChoice = serde_json::from_str(json).unwrap();
    assert!(bc.choice);
    assert_eq!(bc.message, Some("Obligatoire".to_string()));
}
