# Rusti - Core Framework

This is the core library for the Rusti web framework.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
rusti = "0.1"
```

## Features

- `default`: Includes ORM support
- `orm`: SeaORM database integration
- `full`: All features enabled

## Documentation

For full documentation and examples, see the [main repository](../README.md).

## Development

This crate is part of the Rusti workspace. To develop:

```bash
# From workspace root
cargo build
cargo test
cargo doc
```

## API Overview

### Core Types

- `RustiApp`: Main application struct
- `Settings`: Configuration
- `RustiError`: Error types
- `RustiResponse`: Response helpers

### Prelude

Import commonly used types:

```rust
use rusti::prelude::*;
```

This includes:
- Application builders
- Axum essentials
- Tera templating
- Serde for serialization
- Common traits

## Examples

See `examples/` directory in the workspace root.
