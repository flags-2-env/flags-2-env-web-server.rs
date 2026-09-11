#![forbid(unsafe_code)]

use std::{collections::BTreeMap, fmt};

#[derive(Clone)]
pub struct WebConfig {
    pub bind: String,
    pub api_http_base: Option<String>,
    pub database_url: Option<String>,
}

impl fmt::Debug for WebConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebConfig")
            .field("bind", &self.bind)
            .field("api_http_base", &self.api_http_base)
            .field("database_configured", &self.database_url.is_some())
            .finish()
    }
}

impl WebConfig {
    pub fn from_env() -> Result<Self, crate::env::MissingEnv> {
        crate::env::load().map(|environment| Self::from_map(&environment))
    }

    pub fn from_map(environment: &BTreeMap<String, String>) -> Self {
        Self {
            bind: environment
                .get(crate::env::BIND)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8081".to_owned()),
            api_http_base: environment.get(crate::env::API_HTTP_BASE).cloned(),
            database_url: environment.get(crate::env::DATABASE_URL).cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WebConfig;

    #[test]
    fn debug_output_never_contains_database_url() {
        let marker = "postgres://user:synthetic-secret@db.example/flags2env";
        let config = WebConfig {
            bind: "127.0.0.1:8081".to_owned(),
            api_http_base: Some("https://api.flags-2-env.dev".to_owned()),
            database_url: Some(marker.to_owned()),
        };

        let rendered = format!("{config:?}");
        assert!(!rendered.contains(marker));
        assert!(rendered.contains("database_configured: true"));
    }
}
