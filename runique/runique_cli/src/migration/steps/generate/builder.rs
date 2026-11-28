//! The builder of a migration: one method per case, an action then its
//! object, as in `instruction`. Called in any order, each case is put where
//! it runs, and `down` is `up` read backwards.
//!
//! ```rust,ignore
//! BuilderPlan::plan(engine)
//!     .rename_column("books", "genre", "style")
//!     .delete_column("books", &former)   // put last, whatever the order of the calls
//!     .create_enum_value("genre", "Essai")
//!     .build()
//! ```
//!
//! A migration planned for an engine holds what that engine has something
//! to run for and nothing else (`Instruction::runs_on`).
use super::{
    file_around,
    instruction::{Create, Delete, Direction, File, Instruction, Modify, Rename, body},
};
use crate::{
    db::DatabaseEngine,
    migration::types::{ColumnType, EnumPlace, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

pub(super) struct BuilderPlan<'a> {
    engine: Option<&'a DatabaseEngine>,
    instructions: Vec<Instruction<'a>>,
    guard: String,
    when_nothing_runs: String,
}

impl<'a> BuilderPlan<'a> {
    /// A migration written for `engine`, the one of the project.
    pub(super) fn plan(engine: &'a DatabaseEngine) -> Self {
        Self::with(Some(engine))
    }

    /// A migration that holds every case, for a file that is not written
    /// for an engine: what differs between engines is then told apart when
    /// the migration runs.
    pub(super) fn for_every_engine() -> Self {
        Self::with(None)
    }

    fn with(engine: Option<&'a DatabaseEngine>) -> Self {
        Self {
            engine,
            instructions: Vec::new(),
            guard: String::new(),
            when_nothing_runs: String::new(),
        }
    }

    fn case(&mut self, instruction: Instruction<'a>) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    // ── Create ──────────────────────────────────────────────────────────────

    pub(super) fn create_table(
        &mut self,
        schema: &'a ParsedSchema,
        forward: &'a [&'a ParsedFk],
        file: File,
    ) -> &mut Self {
        self.case(Instruction::Create(Create::Table {
            schema,
            forward,
            file,
        }))
    }

    pub(super) fn create_column(&mut self, table: &'a str, column: &'a ParsedColumn) -> &mut Self {
        self.case(Instruction::Create(Create::Column { table, column }))
    }

    pub(super) fn create_key(
        &mut self,
        table: &'a str,
        name: String,
        key: &'a ParsedFk,
        file: File,
    ) -> &mut Self {
        self.case(Instruction::Create(Create::Key {
            table,
            name,
            key,
            file,
        }))
    }

    pub(super) fn create_cycle_keys(&mut self, keys: &'a [(&'a str, &'a ParsedFk)]) -> &mut Self {
        self.case(Instruction::Create(Create::CycleKeys { keys }))
    }

    /// `named_for`: the table name the index is named for.
    pub(super) fn create_index(
        &mut self,
        table: &'a str,
        named_for: &'a str,
        index: &'a ParsedIndex,
        file: File,
    ) -> &mut Self {
        self.case(Instruction::Create(Create::Index {
            table,
            named_for,
            index,
            file,
        }))
    }

    pub(super) fn create_enum_type(&mut self, name: &'a str, values: &'a [String]) -> &mut Self {
        self.case(Instruction::Create(Create::EnumType { name, values }))
    }

    pub(super) fn create_enum_value(&mut self, enum_name: &'a str, value: &'a str) -> &mut Self {
        self.case(Instruction::Create(Create::EnumValue { enum_name, value }))
    }

    // ── Delete ──────────────────────────────────────────────────────────────

    pub(super) fn delete_table(&mut self, name: &'a str, if_exists: bool) -> &mut Self {
        self.case(Instruction::Delete(Delete::Table { name, if_exists }))
    }

    pub(super) fn delete_column(&mut self, table: &'a str, column: &'a ParsedColumn) -> &mut Self {
        self.case(Instruction::Delete(Delete::Column { table, column }))
    }

    pub(super) fn delete_key(
        &mut self,
        table: &'a str,
        name: String,
        key: &'a ParsedFk,
        file: File,
    ) -> &mut Self {
        self.case(Instruction::Delete(Delete::Key {
            table,
            name,
            key,
            file,
        }))
    }

    pub(super) fn delete_cycle_keys(&mut self, keys: &'a [(&'a str, &'a ParsedFk)]) -> &mut Self {
        self.case(Instruction::Delete(Delete::CycleKeys { keys }))
    }

    /// `named_for`: the table name the index is named for.
    pub(super) fn delete_index(
        &mut self,
        table: &'a str,
        named_for: &'a str,
        index: &'a ParsedIndex,
        file: File,
    ) -> &mut Self {
        self.case(Instruction::Delete(Delete::Index {
            table,
            named_for,
            index,
            file,
        }))
    }

    pub(super) fn delete_enum_type(&mut self, name: &'a str, values: &'a [String]) -> &mut Self {
        self.case(Instruction::Delete(Delete::EnumType { name, values }))
    }

    pub(super) fn delete_enum_value(&mut self, enum_name: &'a str, value: &'a str) -> &mut Self {
        self.case(Instruction::Delete(Delete::EnumValue { enum_name, value }))
    }

    // ── Modify ──────────────────────────────────────────────────────────────

    pub(super) fn modify_column(
        &mut self,
        table: &'a str,
        from: &'a ParsedColumn,
        to: &'a ParsedColumn,
    ) -> &mut Self {
        self.case(Instruction::Modify(Modify::Column { table, from, to }))
    }

    pub(super) fn modify_column_type(
        &mut self,
        column: &'a str,
        from: ColumnType,
        to: ColumnType,
    ) -> &mut Self {
        self.case(Instruction::Modify(Modify::ColumnType { column, from, to }))
    }

    pub(super) fn modify_enum_column(
        &mut self,
        table: &'a str,
        from: &'a ParsedColumn,
        to: &'a ParsedColumn,
    ) -> &mut Self {
        self.case(Instruction::Modify(Modify::EnumColumn { table, from, to }))
    }

    // ── Rename ──────────────────────────────────────────────────────────────

    pub(super) fn rename_table(&mut self, from: &'a str, to: &'a str) -> &mut Self {
        self.case(Instruction::Rename(Rename::Table { from, to }))
    }

    pub(super) fn rename_column(
        &mut self,
        table: &'a str,
        from: &'a str,
        to: &'a str,
    ) -> &mut Self {
        self.case(Instruction::Rename(Rename::Column { table, from, to }))
    }

    /// `from` and `to`: the (table, target) names the key is named for,
    /// before and after. `target`: the name its target has at that moment.
    pub(super) fn rename_key(
        &mut self,
        table: &'a str,
        from: (&'a str, &'a str),
        to: (&'a str, &'a str),
        key: &'a ParsedFk,
        target: &'a str,
    ) -> &mut Self {
        self.case(Instruction::Rename(Rename::Key {
            table,
            from,
            to,
            key,
            target,
        }))
    }

    /// `from` and `to`: the index, and the table name it is named for.
    pub(super) fn rename_index(
        &mut self,
        table: &'a str,
        from: (&'a ParsedIndex, &'a str),
        to: (&'a ParsedIndex, &'a str),
    ) -> &mut Self {
        self.case(Instruction::Rename(Rename::Index { table, from, to }))
    }

    pub(super) fn rename_enum_type(&mut self, from: &'a str, to: &'a str) -> &mut Self {
        self.case(Instruction::Rename(Rename::EnumType { from, to }))
    }

    /// Once for the enum: `places` are every column that uses it.
    pub(super) fn rename_enum_value(
        &mut self,
        enum_name: &'a str,
        from: &'a str,
        to: &'a str,
        places: &'a [EnumPlace],
    ) -> &mut Self {
        self.case(Instruction::Rename(Rename::EnumValue {
            enum_name,
            from,
            to,
            places,
        }))
    }

    // ── What surrounds the cases ────────────────────────────────────────────

    /// What both `up` and `down` start with, before any case: a check that
    /// stops the migration while nothing is changed yet.
    pub(super) fn guard(&mut self, check: String) -> &mut Self {
        self.guard = check;
        self
    }

    /// What `up` holds when no case has anything to run on the engine: a
    /// note, the migration being still written for what applying it records.
    pub(super) fn when_nothing_runs(&mut self, note: String) -> &mut Self {
        self.when_nothing_runs = note;
        self
    }

    /// The cases of `up`, each where it runs.
    pub(super) fn up(&self) -> String {
        body(&self.instructions, Direction::Up, self.engine)
    }

    /// The way back: `up` read backwards, each case replaced by its inverse.
    pub(super) fn down(&self) -> String {
        body(&self.instructions, Direction::Down, self.engine)
    }

    /// The migration file.
    pub(super) fn build(&self) -> String {
        let up = self.up();
        let up = if up.is_empty() {
            &self.when_nothing_runs
        } else {
            &up
        };
        file_around(
            &format!("{}{up}", self.guard),
            &format!("{}{}", self.guard, self.down()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{super::instruction::write, *};
    use crate::migration::types::FkAction;

    const PG: DatabaseEngine = DatabaseEngine::PostgreSQL;

    fn column(name: &str) -> ParsedColumn {
        ParsedColumn {
            name: name.into(),
            ..ParsedColumn::default()
        }
    }

    fn key() -> ParsedFk {
        ParsedFk {
            from_column: "author_id".into(),
            to_table: "authors".into(),
            to_column: "id".into(),
            on_delete: FkAction::Cascade,
            on_update: FkAction::NoAction,
        }
    }

    fn index(name: &str) -> ParsedIndex {
        ParsedIndex {
            name: name.into(),
            columns: vec!["title".into()],
            unique: false,
        }
    }

    /// Where `needle` is in `text`.
    fn at(text: &str, needle: &str) -> usize {
        text.find(needle)
            .unwrap_or_else(|| panic!("{needle} in:\n{text}"))
    }

    // Each method is its case and no other: it writes what the instruction
    // of the same action and object writes, with the same values at the same
    // places. Every action and object of `Instruction` is here: 23 cases,
    // a dropped table in its two forms.
    #[test]
    fn each_method_writes_its_own_case() {
        let (title, summary) = (column("title"), column("summary"));
        let (fk, by_title, by_name) = (key(), index("idx_books_title"), index("idx_library_title"));
        let schema = ParsedSchema {
            table_name: "books".into(),
            columns: vec![title.clone()],
            ..ParsedSchema::default()
        };
        let forward = [&fk];
        let cycle = [("books", &fk)];
        let values = vec!["Roman".to_string(), "Policier".to_string()];
        let places = vec![EnumPlace {
            table: "books".into(),
            column: "genre".into(),
        }];
        let name = || "books_author_id_authors_fkey".to_string();

        macro_rules! same {
            ($method:ident($($arg:expr),*), $instruction:expr) => {{
                let mut builder = BuilderPlan::for_every_engine();
                builder.$method($($arg),*);
                let written = write(&$instruction);
                assert!(!written.is_empty(), stringify!($method));
                assert_eq!(builder.up(), written, stringify!($method));
            }};
        }

        same!(
            create_table(&schema, &forward, File::Migration),
            Instruction::Create(Create::Table {
                schema: &schema,
                forward: &forward,
                file: File::Migration
            })
        );
        same!(
            create_column("books", &title),
            Instruction::Create(Create::Column {
                table: "books",
                column: &title
            })
        );
        same!(
            create_key("books", name(), &fk, File::Migration),
            Instruction::Create(Create::Key {
                table: "books",
                name: name(),
                key: &fk,
                file: File::Migration
            })
        );
        same!(
            create_cycle_keys(&cycle),
            Instruction::Create(Create::CycleKeys { keys: &cycle })
        );
        same!(
            create_index("library", "books", &by_title, File::Migration),
            Instruction::Create(Create::Index {
                table: "library",
                named_for: "books",
                index: &by_title,
                file: File::Migration
            })
        );
        same!(
            create_enum_type("genre", &values),
            Instruction::Create(Create::EnumType {
                name: "genre",
                values: &values
            })
        );
        same!(
            create_enum_value("genre", "Essai"),
            Instruction::Create(Create::EnumValue {
                enum_name: "genre",
                value: "Essai"
            })
        );
        same!(
            delete_table("books", true),
            Instruction::Delete(Delete::Table {
                name: "books",
                if_exists: true
            })
        );
        same!(
            delete_table("books", false),
            Instruction::Delete(Delete::Table {
                name: "books",
                if_exists: false
            })
        );
        same!(
            delete_column("books", &title),
            Instruction::Delete(Delete::Column {
                table: "books",
                column: &title
            })
        );
        same!(
            delete_key("books", name(), &fk, File::Migration),
            Instruction::Delete(Delete::Key {
                table: "books",
                name: name(),
                key: &fk,
                file: File::Migration
            })
        );
        same!(
            delete_cycle_keys(&cycle),
            Instruction::Delete(Delete::CycleKeys { keys: &cycle })
        );
        same!(
            delete_index("library", "books", &by_title, File::Migration),
            Instruction::Delete(Delete::Index {
                table: "library",
                named_for: "books",
                index: &by_title,
                file: File::Migration
            })
        );
        same!(
            delete_enum_type("genre", &values),
            Instruction::Delete(Delete::EnumType {
                name: "genre",
                values: &values
            })
        );
        same!(
            delete_enum_value("genre", "Essai"),
            Instruction::Delete(Delete::EnumValue {
                enum_name: "genre",
                value: "Essai"
            })
        );
        same!(
            modify_column("books", &title, &summary),
            Instruction::Modify(Modify::Column {
                table: "books",
                from: &title,
                to: &summary
            })
        );
        same!(
            modify_column_type("title", ColumnType::String, ColumnType::Integer),
            Instruction::Modify(Modify::ColumnType {
                column: "title",
                from: ColumnType::String,
                to: ColumnType::Integer
            })
        );
        same!(
            modify_enum_column("books", &title, &summary),
            Instruction::Modify(Modify::EnumColumn {
                table: "books",
                from: &title,
                to: &summary
            })
        );
        same!(
            rename_table("books", "library"),
            Instruction::Rename(Rename::Table {
                from: "books",
                to: "library"
            })
        );
        same!(
            rename_column("books", "title", "name"),
            Instruction::Rename(Rename::Column {
                table: "books",
                from: "title",
                to: "name"
            })
        );
        same!(
            rename_key(
                "library",
                ("books", "authors"),
                ("library", "writers"),
                &fk,
                "authors"
            ),
            Instruction::Rename(Rename::Key {
                table: "library",
                from: ("books", "authors"),
                to: ("library", "writers"),
                key: &fk,
                target: "authors"
            })
        );
        same!(
            rename_index("library", (&by_title, "books"), (&by_name, "library")),
            Instruction::Rename(Rename::Index {
                table: "library",
                from: (&by_title, "books"),
                to: (&by_name, "library")
            })
        );
        same!(
            rename_enum_type("genre", "category"),
            Instruction::Rename(Rename::EnumType {
                from: "genre",
                to: "category"
            })
        );
        same!(
            rename_enum_value("genre", "Roman", "Novel", &places),
            Instruction::Rename(Rename::EnumValue {
                enum_name: "genre",
                from: "Roman",
                to: "Novel",
                places: &places
            })
        );
    }

    // Called last first: each case is still put where it runs, what destroys
    // data closing the migration.
    #[test]
    fn the_cases_run_in_their_order_whatever_the_order_of_the_calls() {
        let (former, fresh) = (column("former"), column("fresh"));
        let mut plan = BuilderPlan::for_every_engine();
        plan.delete_column("books", &former)
            .create_column("books", &fresh)
            .rename_column("books", "title", "name")
            .rename_table("shelf", "books");
        let up = plan.up();

        let (table, renamed, added, dropped) = (
            at(&up, "rename_table"),
            at(&up, "rename_column"),
            at(&up, "add_column"),
            at(&up, "drop_column"),
        );
        assert!(
            table < renamed && renamed < added && added < dropped,
            "{up}"
        );
    }

    // `down` is `up` backwards, each case replaced by its inverse: what was
    // dropped comes back first, the table takes its former name last.
    #[test]
    fn down_is_up_read_backwards() {
        let former = column("former");
        let mut plan = BuilderPlan::for_every_engine();
        plan.rename_table("shelf", "books")
            .delete_column("books", &former);
        let down = plan.down();

        let back = at(&down, r#".add_column(ColumnDef::new(Alias::new("former"))"#);
        let renamed = at(&down, "rename_table");
        assert!(back < renamed, "{down}");
        assert!(!down.contains("drop_column"), "{down}");
        // The table goes back from its new name to the former one.
        assert!(at(&down, r#"Alias::new("books")"#) < at(&down, r#"Alias::new("shelf")"#));
    }

    // Planned for an engine, the migration holds what that engine has
    // something to run for; for every engine, all of it.
    #[test]
    fn a_plan_for_an_engine_holds_what_that_engine_runs() {
        let values = vec!["Roman".to_string()];
        fn fill<'a>(plan: &mut BuilderPlan<'a>, values: &'a [String]) {
            plan.create_enum_type("genre", values).rename_enum_value(
                "genre",
                "Roman",
                "Novel",
                &[],
            );
        }
        let holds = |text: &str| {
            (
                text.contains("create_enum_type"),
                text.contains("rename_enum_value"),
            )
        };

        let mut every = BuilderPlan::for_every_engine();
        fill(&mut every, &values);
        assert_eq!(holds(&every.up()), (true, true));

        let mut postgres = BuilderPlan::plan(&PG);
        fill(&mut postgres, &values);
        assert_eq!(holds(&postgres.up()), (true, true));

        for engine in [DatabaseEngine::MySQL, DatabaseEngine::SQLite] {
            let mut text = BuilderPlan::plan(&engine);
            fill(&mut text, &values);
            assert_eq!(holds(&text.up()), (false, true), "{engine:?}");
            // The way back is filtered too: no type to drop there.
            assert!(!text.down().contains("drop_enum_type"), "{engine:?}");
        }
        assert!(postgres.down().contains("drop_enum_type"));
    }

    // The guard opens both bodies, before any case.
    #[test]
    fn the_guard_opens_both_bodies() {
        let fresh = column("fresh");
        let mut plan = BuilderPlan::for_every_engine();
        plan.create_column("books", &fresh)
            .guard("        // CHECK FIRST\n".to_string());
        let file = plan.build();
        let (up, down) = file.split_once("async fn down").unwrap();

        assert!(at(up, "// CHECK FIRST") < at(up, "add_column"), "{up}");
        assert!(
            at(down, "// CHECK FIRST") < at(down, "drop_column"),
            "{down}"
        );
        assert_eq!(file.matches("// CHECK FIRST").count(), 2);
    }

    // The note stands in for `up` only when nothing runs there.
    #[test]
    fn the_note_is_written_only_when_nothing_runs() {
        let values = vec!["Roman".to_string()];
        let note = || "        // NOTHING HERE\n".to_string();

        let engine = DatabaseEngine::SQLite;
        let mut nothing = BuilderPlan::plan(&engine);
        nothing
            .create_enum_type("genre", &values)
            .when_nothing_runs(note());
        let file = nothing.build();
        assert!(file.contains("// NOTHING HERE"), "{file}");
        assert!(file.contains("_manager: &SchemaManager"), "{file}");

        let mut something = BuilderPlan::plan(&PG);
        something
            .create_enum_type("genre", &values)
            .when_nothing_runs(note());
        let file = something.build();
        assert!(!file.contains("// NOTHING HERE"), "{file}");
        assert!(file.contains("create_enum_type"), "{file}");

        // Without a note, a migration where nothing runs is still a file.
        let mut silent = BuilderPlan::plan(&engine);
        silent.create_enum_type("genre", &values);
        let file = silent.build();
        assert!(file.contains("async fn up(&self, _manager"), "{file}");
    }
}
