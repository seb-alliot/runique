//! `RuniqueTest`: runs business-logic tests against a real database, each one
//! inside its own transaction, always rolled back.
use super::struct_test::{FormatResult, Reason, RuniqueTest, SetupError, TraceSink};
use super::transaction_test::TestTransaction;
use std::{
    process::ExitCode,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

impl<C: TestTransaction> RuniqueTest<C> {
    /// Loads `env_file` — its values override anything already set, so the
    /// file named here always decides which database is used — then connects.
    pub async fn new(env_file: &str) -> Result<Self, SetupError<C::Error>> {
        dotenvy::from_filename_override(env_file)
            .map_err(|e| SetupError::EnvFile(format!("{env_file}: {e}")))?;
        let config = C::load_config().map_err(SetupError::Engine)?;
        Self::with_config(&config).await.map_err(SetupError::Engine)
    }

    /// Connects with an explicit config, without reading any env file.
    pub async fn with_config(config: &C::Config) -> Result<Self, C::Error> {
        println!("runique test → {}", C::describe(config));
        let trace = TraceSink::default();
        let connection = C::connect(config, trace.clone()).await?;
        Ok(Self {
            connection,
            trace,
            filter: None,
            seen: Mutex::default(),
            passed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            serial: tokio::sync::Mutex::new(()),
        })
    }

    /// Runs only the test named `name`; `None` runs them all.
    pub fn filter(mut self, name: Option<String>) -> Self {
        self.filter = name;
        self
    }

    /// Runs `handler` in a fresh transaction, prints the verdict and the SQL
    /// trace, then rolls back. `handler` only borrows the connection, so the
    /// builder owns it again afterwards and can roll back.
    pub async fn run<F>(&self, name: &str, handler: F)
    where
        F: AsyncFnOnce(&C) -> Result<(), C::Error>,
    {
        self.seen
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(name.to_string());
        if let Some(filter) = &self.filter
            && filter != name
        {
            return;
        }

        let _serial = self.serial.lock().await;
        self.trace.lock().unwrap_or_else(|e| e.into_inner()).clear();
        println!("▶ {name}");

        let mut rollback_error = None;
        let reason = match C::begin_test(&self.connection).await {
            Err(e) => Reason::Error(e),
            Ok(db) => {
                let outcome = handler(&db).await;
                if let Err(e) = db.rollback_test().await {
                    rollback_error = Some(e.to_string());
                }
                match outcome {
                    Ok(()) => Reason::Win,
                    Err(e) => Reason::Error(e),
                }
            }
        };

        let result = FormatResult {
            name_test: name.to_string(),
            reason,
            trace: std::mem::take(&mut *self.trace.lock().unwrap_or_else(|e| e.into_inner())),
            rollback_error,
        };
        print!("{result}");
        let counter = if result.passed() {
            &self.passed
        } else {
            &self.failed
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Prints the summary. Fails if a test failed, or if the requested test
    /// does not exist — then listing the ones that do.
    pub fn finish(self) -> ExitCode {
        let seen = self.seen.into_inner().unwrap_or_else(|e| e.into_inner());
        if let Some(filter) = &self.filter
            && !seen.iter().any(|name| name == filter)
        {
            eprintln!("✗ no test named \"{filter}\"");
            eprintln!("  available tests: {}", seen.join(", "));
            return ExitCode::FAILURE;
        }
        let passed = self.passed.into_inner();
        let failed = self.failed.into_inner();
        println!("\n{passed} passed, {failed} failed");
        if failed == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}
