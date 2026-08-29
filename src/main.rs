#![forbid(unsafe_code)]

use flags_2_env_web_server::{config::WebConfig, flags, server};

fn main() {
    let environment = flags::resolve().unwrap_or_else(|error| panic!("{error}"));
    let cfg = WebConfig::from_map(&environment);
    server::run(&cfg);
}
