// src/env/env.rs — service overlay. Edit defaults here; regenerate generated.rs from flags-2-env.

use super::generated;

pub const DATABASE_URL: &str = "FLAGS_2_ENV_DATABASE_URL";

/// Code-level defaults. Generated flags-2-env values override these defaults.
pub fn defaults() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([(
        "FLAGS_2_ENV_WEB_BIND".to_string(),
        "127.0.0.1:8081".to_string(),
    )])
}

/// Merge service defaults under the generated flags-2-env projection, then add
/// the database DSN from the process environment only. `.cli-flags.toml` uses
/// `files = []`, so long-running server configuration never depends on dotenv.
pub fn load() -> Result<std::collections::BTreeMap<String, String>, generated::MissingEnv> {
    let mut merged = defaults();
    merged.extend(generated::load_env_map_from_os()?);
    if let Ok(database_url) = std::env::var(DATABASE_URL) {
        let database_url = database_url.trim();
        if !database_url.is_empty() {
            merged.insert(DATABASE_URL.to_owned(), database_url.to_owned());
        }
    }
    Ok(merged)
}

pub fn get<'a>(env: &'a std::collections::BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    env.get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
