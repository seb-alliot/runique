-- Runs once, when the Postgres volume is created: the demo's database, next to
-- runique_test (the tests' one, from POSTGRES_DB).
CREATE DATABASE runique_demo OWNER runique;
