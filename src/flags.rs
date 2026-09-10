//! Fail-closed argv and environment resolution through flags-2-env.

use std::collections::BTreeMap;
use std::io::Write;

use flags2env::BundledFlags2Env;
use tempfile::NamedTempFile;

const CONTRACT: &str = include_str!("../.cli-flags.toml");
const ENV_ONLY_KEYS: &[&str] = &["FLAGS_2_ENV_DATABASE_URL"];

pub fn resolve() -> Result<BTreeMap<String, String>, String> {
    resolve_from(&std::env::args().collect::<Vec<_>>(), std::env::vars())
}

fn resolve_from(
    argv: &[String],
    environment: impl IntoIterator<Item = (String, String)>,
) -> Result<BTreeMap<String, String>, String> {
    let mut contract = NamedTempFile::new()
        .map_err(|error| format!("cannot create embedded flags-2-env contract: {error}"))?;
    contract
        .write_all(CONTRACT.as_bytes())
        .map_err(|error| format!("cannot materialize embedded flags-2-env contract: {error}"))?;
    let path = contract
        .path()
        .to_str()
        .ok_or_else(|| "flags-2-env contract path is not valid UTF-8".to_owned())?;
    let parser = BundledFlags2Env::new();
    parser
        .audit_config(Some(path))
        .map_err(|error| format!("flags-2-env contract audit failed: {error}"))?;
    let parsed = parser
        .parse_structured(argv, Some(path))
        .map_err(|error| format!("flags-2-env parsing failed: {error}"))?;

    if !parsed.unknown_options.is_empty() {
        let names = parsed
            .unknown_options
            .iter()
            .map(|option| option.split('=').next().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("unknown command-line option(s): {names}"));
    }
    if !parsed.errors.is_empty() {
        return Err(format!(
            "invalid command-line value(s): {}",
            parsed.errors.join("; ")
        ));
    }
    if !parsed.extras.is_empty() {
        return Err(format!(
            "unexpected positional argument(s): {}",
            parsed.extras.len()
        ));
    }

    let environment = environment.into_iter().collect::<BTreeMap<_, _>>();
    let mut raw = parsed.dotenv;
    raw.extend(environment);
    raw.extend(parsed.dotenv_overrides);
    raw.extend(parsed.provided_flags);

    let typed = parser
        .coerce::<serde_json::Map<String, serde_json::Value>, _>(&raw, Some(path))
        .map_err(|error| format!("flags-2-env typed configuration failed: {error}"))?;
    let typed_entries = typed
        .into_iter()
        .filter(|(_, value)| !value.is_null())
        .map(|(name, value)| scalar_string(&name, value).map(|value| (name, value)));
    let environment_only = ENV_ONLY_KEYS.iter().filter_map(|name| {
        raw.get(*name)
            .map(|value| Ok::<(String, String), String>(((*name).to_owned(), value.clone())))
    });

    typed_entries.chain(environment_only).collect()
}

fn scalar_string(name: &str, value: serde_json::Value) -> Result<String, String> {
    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Bool(value) => Ok(value.to_string()),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err(format!(
            "flags-2-env returned a non-scalar value for {name}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_options_fail_closed_without_echoing_values() -> Result<(), String> {
        let error = match resolve_from(
            &[
                "server".to_owned(),
                "--definitely-unknown=do-not-echo".to_owned(),
            ],
            std::iter::empty(),
        ) {
            Ok(_) => return Err("unknown option was unexpectedly accepted".to_owned()),
            Err(error) => error,
        };
        assert!(error.contains("--definitely-unknown"));
        assert!(!error.contains("do-not-echo"));
        Ok(())
    }

    #[test]
    fn database_url_is_environment_only_and_preserved() -> Result<(), String> {
        let marker = "postgres://user:synthetic-secret@127.0.0.1:5432/flags2env";
        let resolved = resolve_from(
            &["server".to_owned()],
            [("FLAGS_2_ENV_DATABASE_URL".to_owned(), marker.to_owned())],
        )?;

        assert_eq!(
            resolved.get("FLAGS_2_ENV_DATABASE_URL").map(String::as_str),
            Some(marker)
        );
        Ok(())
    }

    #[test]
    fn database_url_cannot_be_supplied_on_argv_or_reflected() -> Result<(), String> {
        let marker = "synthetic-secret-never-reflect";
        let error = match resolve_from(
            &[
                "server".to_owned(),
                format!("--database-url={marker}"),
            ],
            std::iter::empty(),
        ) {
            Ok(_) => return Err("database URL was unexpectedly argv-addressable".to_owned()),
            Err(error) => error,
        };

        assert!(error.contains("--database-url"));
        assert!(!error.contains(marker));
        Ok(())
    }
}
