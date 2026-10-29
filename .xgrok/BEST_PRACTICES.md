# Best practices (project-aware)

Stack signals: **Cargo**  
Languages: **Rust**

## For humans + Grok
- Small, reviewable changes.
- Tests for non-trivial logic.
- Document public entrypoints and env vars in README.
- Keep AGENTS.md in sync when layout/commands change.

## Stack notes



- `cargo fmt` / clippy mindset; no unwrap in production paths without reason.
