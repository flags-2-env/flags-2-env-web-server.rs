#![forbid(unsafe_code)]

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub bind: String,
    pub api_http_base: Option<String>,
    pub database_url: Option<String>,
}

impl WebConfig {
    pub fn from_env() -> Self {
        let env = crate::env::load().unwrap_or_else(|err| panic!("{err}"));
        Self {
            bind: crate::env::get(&env, crate::env::BIND)
                .unwrap_or("127.0.0.1:8081")
                .to_owned(),
            api_http_base: crate::env::get(&env, crate::env::API_HTTP_BASE).map(str::to_owned),
            database_url: crate::env::get(&env, crate::env::DATABASE_URL).map(str::to_owned),
        }
    }
}
