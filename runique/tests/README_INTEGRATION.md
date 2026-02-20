# 🧪 Integration Tests for Runique

Integration test suite for the Runique framework.

## 📁 Structure

```text
tests/
├── integration_tests.rs  # Main integration tests
└── README.md            # Test documentation
```

## 🧪 Running the tests

c

### Integration tests

```bash
cargo test --test integration_tests
```


### Library unit tests

```bash
cargo test --lib
```


### All tests

```bash
cargo test --all
```

## 📊 Test statistics

| Type | Number | Status |
| ------ | -------- | -------- |
| Integration tests | 16 | ✅ Passing |
| Unit tests | 20 | ✅ Passing |
| **Total** | **36** | **✅ All passing** |

## 🧪 Available tests

### Form tests (8 tests)

- `test_text_field_creation` - Text field creation
- `test_email_field` - Email field creation
- `test_password_field` - Password field creation
- `test_textarea_field` - Textarea field creation
- `test_richtext_field` - Rich text field creation
- `test_url_field` - URL field creation
- `test_numeric_field_integer` - Integer numeric field creation
- `test_numeric_field_decimal` - Decimal numeric field creation

### Pattern builder tests (1 test)

- `test_text_field_builder` - Pattern builder test for fields

### Required fields tests (1 test)

- `test_field_required` - Required fields test

### Form management tests (4 tests)

- `test_forms_new` - Create a form with CSRF token
- `test_forms_add_field` - Add a field to a form
- `test_forms_fill_data` - Fill a form with data
- `test_complex_form_creation` - Create a complex form

### Configuration tests (2 tests)

- `test_prelude_exports` - Check that structures are available
- `test_field_types_available` - Check that all types are available

## ✅ Validated points

- ✅ Framework compiles without errors
- ✅ All form types work
- ✅ Pattern builder works correctly
- ✅ Forms accept multiple fields
- ✅ Basic validation works
- ✅ Integration with SeaORM works

## 📝 Usage example

```rust
#[test]
fn example_test() {
    use runique::prelude::*;

    let mut form = Forms::new("csrf_token");
    form.field(&TextField::text("username")
        .label("Username"));

    assert!(form.fields.contains_key("username"));
}
```

## 🔧 Useful commands

Run a specific test:

```bash
cargo test --test integration_tests test_text_field_creation -- --nocapture
```

Run avec backtrace :

```bash
RUST_BACKTRACE=1 cargo test --test integration_tests
```

Run en mode verbose :

```bash
cargo test --test integration_tests -- --nocapture
```
