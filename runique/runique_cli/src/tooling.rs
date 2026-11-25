//! What the command knows of the framework it works for. Copied from
//! `runique::tooling`, where these values are read from the crate itself:
//! here they are written, and the tests check they still are the framework's.

/// The version of `runique`: the one `runique new` and `makemigrations` write
/// as the `runique` dependency of a project.
pub const VERSION: &str = "4.1.0-preview";

/// The SeaORM requirements `runique` is built with: a generated project pins
/// the same.
pub const SEA_ORM_REQ: &str = "=2.0.4";
pub const SEA_ORM_MIGRATION_REQ: &str = "=2.0.4";

/// Set by `runique test` when it writes to a terminal. Its output goes through
/// a pipe, so the test code can't tell it's a terminal and would drop colors.
pub const TEST_COLOR_ENV: &str = "RUNIQUE_TEST_COLOR";

#[cfg(test)]
mod tests {
    // A new version of `runique`, or of SeaORM in it, must be written here
    // too: this is the test that says so.
    #[test]
    fn the_values_are_those_of_runique() {
        assert_eq!(super::VERSION, runique::tooling::VERSION);
        assert_eq!(super::SEA_ORM_REQ, runique::tooling::SEA_ORM_REQ);
        assert_eq!(
            super::SEA_ORM_MIGRATION_REQ,
            runique::tooling::SEA_ORM_MIGRATION_REQ
        );
        assert_eq!(super::TEST_COLOR_ENV, runique::tooling::TEST_COLOR_ENV);
    }
}
