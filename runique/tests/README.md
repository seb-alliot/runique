tests/
# 🧪 Runique Framework Tests

Comprehensive test suite covering all main features of the Runique framework.

## 📁 Test Structure

```

├── common.rs              # Shared utilities and helpers
├── forms_test.rs          # Form system tests (Prisme extractor)
├── orm_test.rs            # ORM tests (Objects manager, impl_objects!)
├── config_test.rs         # Configuration tests
├── flash_messages_test.rs # Flash message tests (Message extractor)
├── routes_test.rs         # Routing tests (Axum Router)
├── middleware_test.rs     # Middleware tests (CSRF, CSP, etc.)
├── prelude_test.rs        # Tests that all types are in the prelude
└── README.md              # This file
```

## 🧪 Running the Tests

### All tests
```bash
cargo test
```

### A specific test file
```bash
cargo test --test forms_test
cargo test --test macros_test
cargo test --test orm_test
```

### A specific test
```bash
cargo test test_text_field_creation
cargo test test_forms_new
cargo test test_context_macro_empty
```

### With output
```bash
cargo test -- --nocapture
cargo test -- --show-output
```

## 📋 Test Coverage

### ✅ Forms (`forms_test.rs`)
- [x] Prisme extractor
- [x] RuniqueForm derive macro
- [x] Field validation
- [x] HTML generation for forms
- [x] CSRF token validation

### ✅ ORM (`orm_test.rs`)
- [x] impl_objects! macro
- [x] Objects manager (.all(), .filter(), etc.)
- [x] SeaORM integration
- [x] Relations

### ✅ Flash Messages (`flash_messages_test.rs`)
- [x] Message extractor
- [x] success(), error(), info(), warning() methods
- [x] message.level (Success/Error/Info/Warning)
- [x] {% messages %} template tag

### ✅ Forms (`forms_test.rs`)
- [x] `TextField` - text, email, password, textarea, richtext, url
- [x] `NumericField` - integer, decimal
- [x] `BooleanField`
- [x] Validations - required, min_length, max_length
- [x] `Forms` manager - new, add_field, fill_data
- [x] Method chaining

### ✅ ORM (`orm_test.rs`)
- [x] `Objects<E>` manager
- [x] Chainable methods (filter, exclude, limit, offset, etc.)
- [x] RuniqueQueryBuilder
- [x] Django-style queries

### ✅ Configuration (`config_test.rs`)
- [x] `RuniqueConfig`
- [x] `ServerConfig`
- [x] `SecurityConfig`
- [x] Loading from `.env`

### ✅ Flash Messages (`flash_messages_test.rs`)
- [x] `MessageLevel` (Success, Error, Warning, Info)
- [x] `FlashMessage`
- [x] `Message` type
- [x] Creation and management

### ✅ Routing (`routes_test.rs`)
- [x] `urlpatterns!` macro
- [x] `view!` macro
- [x] HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS)
- [x] `register_name_url` for URL naming
- [x] `reverse` and `reverse_with_parameters`

### ✅ Middlewares (`middleware_test.rs`)
- [x] CSRF middleware
- [x] CSP middleware
- [x] Sanitizer middleware
- [x] Session middleware
- [x] AllowedHosts middleware

### ✅ Prelude (`prelude_test.rs`)
- [x] Available form types
- [x] Available context types
- [x] Available flash message types
- [x] Available ORM types
- [x] Serialization types
- [x] Concurrency types

## 🧩 Common Utilities

The `common.rs` file provides reusable helpers:

```rust
// Create a simple test form
let form = create_test_form("csrf_token");

// Create a complex form with multiple fields
let form = create_complex_form("csrf_token");

// Fill a form with data
fill_form(&mut form, &[("field", "value")]);
```

## 🔧 Adding New Tests

### Template for a new test
```rust
#[test]
fn test_my_feature() {
    // Arrange - Prepare data

    // Act - Execute the code to test

    // Assert - Check the results
    assert!(true);
}
```

### Asynchronous tests
```rust
#[tokio::test]
async fn test_async_feature() -> Result<(), Box<dyn std::error::Error>> {
    // Async code here
    Ok(())
}
```

## ℹ️ Important Notes

1. **Integration tests with DB**: To test the ORM with a real DB, use in-memory SQLite:
   ```rust
   let db = sea_orm::Database::connect("sqlite::memory:").await?;
   ```

2. **Web handler tests**: Use `axum-test` or similar to test Axum handlers

3. **Template tests**: Use `tera` directly to test rendering

4. **Mocking**: For external dependencies, consider `mockito` or `wiremock`

## 📚 Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/tutorial/select#testing)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)

## 🎯 Future Goals

- [ ] Full integration tests with SQLite DB
- [ ] Web handler tests with `axum-test`
- [ ] Tera rendering tests
- [ ] Performance benchmarks
- [ ] Coverage reporting tests
