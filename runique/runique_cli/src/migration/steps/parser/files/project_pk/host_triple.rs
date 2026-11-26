use super::*;

/// The platform this machine builds for, as `rustc` names it.
pub(super) fn host_triple() -> Option<String> {
    let output = Command::new("rustc").arg("-vV").output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_string)
}
