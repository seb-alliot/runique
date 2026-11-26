use super::*;

/// Asks cargo for the project of the current folder.
pub(crate) fn from_cargo() -> Result<PkType> {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--format-version", "1"]);
    // Without it cargo resolves every platform, and fetches crates no build
    // of this machine needs.
    if let Some(host) = host_triple() {
        command.args(["--filter-platform", &host]);
    }
    let output = command.output().map_err(|e| {
        anyhow!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[e.to_string()]
        ))
    })?;
    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[reason]
        ));
    }
    pk_of(&String::from_utf8_lossy(&output.stdout))
}
