use super::*;

/// The features of `runique` that are on in the project, in cargo's
/// description of it (`cargo metadata`): the ones the project asks for and
/// the ones they bring with them.
pub(crate) fn features_of(metadata: &str) -> Result<Vec<String>> {
    let missing = || {
        anyhow!(t(runique_i18n::keys::makemigrations::error::PROJECT_PK_NO_RUNIQUE).into_owned())
    };
    let json: serde_json::Value = serde_json::from_str(metadata).map_err(|e| {
        anyhow!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[e.to_string()]
        ))
    })?;
    let id = json["packages"]
        .as_array()
        .and_then(|packages| {
            packages
                .iter()
                .find(|package| package["name"].as_str() == Some("runique"))
        })
        .and_then(|package| package["id"].as_str())
        .ok_or_else(missing)?;
    let features = json["resolve"]["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["id"].as_str() == Some(id)))
        .and_then(|node| node["features"].as_array())
        .ok_or_else(missing)?;
    Ok(features
        .iter()
        .filter_map(|feature| feature.as_str().map(str::to_string))
        .collect())
}
