#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub bind: String,
    pub api_http_base: Option<String>,
    pub database_url: Option<String>,
}

impl WebConfig {
    pub fn from_env() -> Self {
        let env = crate::env::load().unwrap_or_else(|err| panic!("{err}"));
        Self::from_map(&env)
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
