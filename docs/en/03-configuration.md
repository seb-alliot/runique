# ⚙️ Configuration

## RuniqueConfig Structure

Main configuration object loaded from `.env`:

```rust
pub struct RuniqueConfig {
    pub app_name: String,
    pub secret_key: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub database_url: String,
    pub max_connections: u32,
    pub session_expiry: u64,
}
```

---

## Environment Variables

Create a `.env` file in your project root:

```bash
# Application
APP_NAME=Runique App
SECRET_KEY=your-secret-key-here-min-32-chars
DEBUG=true

# Security
ALLOWED_HOSTS=localhost,127.0.0.1,yourdomain.com

# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/runique
MAX_CONNECTIONS=5

# Session
SESSION_EXPIRY_HOURS=24
```

---

## Configuration Table

| Variable | Type | Default | Purpose |
|----------|------|---------|---------|
| `APP_NAME` | String | "Runique App" | Application name |
| `SECRET_KEY` | String | (required) | Encryption key for sessions/tokens (min 32 chars) |
| `DEBUG` | bool | false | Enable debug mode |
| `ALLOWED_HOSTS` | CSV | localhost | Allowed host headers |
| `DATABASE_URL` | String | (required) | Database connection string |
| `MAX_CONNECTIONS` | u32 | 5 | Database pool size |
| `SESSION_EXPIRY_HOURS` | u64 | 24 | Session timeout in hours |

---

## Loading Configuration

### Automatic Loading

```rust
use runique::config_runique::RuniqueConfig;

// Loads from .env or environment variables
let config = RuniqueConfig::from_env();
```

### Manual Loading

```rust
use dotenv::dotenv;

fn main() {
    dotenv().ok();  // Load .env file
    
    let config = RuniqueConfig::from_env();
    
    if config.debug {
        println!("🔍 Debug mode enabled");
    }
}
```

---

## Complete .env Example

```bash
# ============================================
# RUNIQUE CONFIGURATION
# ============================================

# Application Settings
APP_NAME=My Runique App
DEBUG=true

# SECURITY - Generate with: openssl rand -base64 32
SECRET_KEY=your-secret-key-here-minimum-32-characters

# Allowed hosts (comma-separated)
ALLOWED_HOSTS=localhost,127.0.0.1,0.0.0.0

# ============================================
# DATABASE
# ============================================

# PostgreSQL
DATABASE_URL=postgres://postgres:password@localhost:5432/runique
MAX_CONNECTIONS=10

# Or SQLite
# DATABASE_URL=sqlite:./data.db

# ============================================
# SESSION
# ============================================

SESSION_EXPIRY_HOURS=24

# ============================================
# SMTP (Optional)
# ============================================

# SMTP_HOST=smtp.gmail.com
# SMTP_PORT=587
# SMTP_USER=your-email@gmail.com
# SMTP_PASSWORD=your-app-password
```

---

## Advanced Configuration

### Accessing Config in Code

```rust
use runique::config_runique::RuniqueConfig;

async fn handler(ctx: RuniqueContext) -> Response {
    let config = &ctx.engine.config;
    
    if config.debug {
        println!("App: {}", config.app_name);
        println!("Hosts: {:?}", config.allowed_hosts);
    }
}
```

### Dynamic Configuration

```rust
// Load custom config
pub struct CustomConfig {
    pub base_config: RuniqueConfig,
    pub custom_value: String,
}

impl CustomConfig {
    pub fn from_env() -> Self {
        let base_config = RuniqueConfig::from_env();
        let custom_value = std::env::var("CUSTOM_VALUE")
            .unwrap_or_else(|_| "default".to_string());
        
        CustomConfig {
            base_config,
            custom_value,
        }
    }
}
```

### Validation

```rust
impl RuniqueConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Validate secret key length
        if self.secret_key.len() < 32 {
            return Err("SECRET_KEY must be at least 32 characters".to_string());
        }
        
        // Validate database URL format
        if !self.database_url.starts_with("postgres://") 
            && !self.database_url.starts_with("sqlite:") {
            return Err("DATABASE_URL invalid format".to_string());
        }
        
        // Validate max connections
        if self.max_connections == 0 || self.max_connections > 100 {
            return Err("MAX_CONNECTIONS must be between 1 and 100".to_string());
        }
        
        Ok(())
    }
}
```

---

## Generating Secret Key

### Using OpenSSL

```bash
openssl rand -base64 32
```

Output:
```
aBcDeFgHiJkLmNoPqRsTuVwXyZ1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p
```

Copy this into your `.env`:
```bash
SECRET_KEY=aBcDeFgHiJkLmNoPqRsTuVwXyZ1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p
```

### Using Rust

```rust
use rand::Rng;

fn generate_secret_key() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rng.gen())
        .collect();
    
    base64::encode(&random_bytes)
}
```

---

## Database Configuration

### PostgreSQL

```bash
# .env
DATABASE_URL=postgres://username:password@localhost:5432/database_name
MAX_CONNECTIONS=10
```

**Connection String Format**:
```
postgres://[user[:password]@][host][:port][/database][?key=value...]
```

**Example with credentials**:
```bash
DATABASE_URL=postgres://appuser:secure_password@db.example.com:5432/my_app_db
```

### SQLite

```bash
# .env
DATABASE_URL=sqlite:./data.db
MAX_CONNECTIONS=1
```

---

## Environment-Specific Configs

### Development

```bash
DEBUG=true
ALLOWED_HOSTS=localhost,127.0.0.1
DATABASE_URL=postgres://postgres:password@localhost:5432/runique_dev
SESSION_EXPIRY_HOURS=24
```

### Production

```bash
DEBUG=false
ALLOWED_HOSTS=yourdomain.com,www.yourdomain.com
DATABASE_URL=postgres://appuser:securepass@prod-db:5432/runique
MAX_CONNECTIONS=50
SESSION_EXPIRY_HOURS=1
```

### Testing

```bash
DEBUG=true
ALLOWED_HOSTS=localhost
DATABASE_URL=sqlite:memory:
SESSION_EXPIRY_HOURS=1
```

---

## Configuration in Code

```rust
use runique::prelude::*;

pub async fn run_server(config: RuniqueConfig) -> Result<()> {
    // Validate config
    config.validate()?;
    
    // Setup logging
    if config.debug {
        println!("🚀 Starting {} in debug mode", config.app_name);
    }
    
    // Create database connection
    let db = Database::connect(&config.database_url).await?;
    
    // Setup engine
    let engine = Arc::new(RuniqueEngine {
        db: Arc::new(db),
        config,
    });
    
    // Start server
    let app = build_app(engine);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

---

## Gotchas & Tips

### 1. Secret Key Length
Must be at least 32 characters. Generate with `openssl rand -base64 32`

### 2. DATABASE_URL Format
- PostgreSQL: `postgres://user:pass@host:port/db`
- SQLite: `sqlite:./path/to/file.db` or `sqlite::memory:`

### 3. Allowed Hosts
Comma-separated list without spaces:
```bash
ALLOWED_HOSTS=localhost,127.0.0.1,yourdomain.com
```

### 4. Session Expiry
Set to 0 for no expiry (not recommended):
```bash
SESSION_EXPIRY_HOURS=24
```

### 5. .env in Git
Add to `.gitignore`:
```bash
.env
.env.local
.env.*.local
```

---

## Next Steps

← [**Installation**](./01-installation.md) | [**Routing**](./04-routing.md) →
