#![forbid(unsafe_code)]

use flags_2_env_web_server::{config::WebConfig, server};

fn main() {
    let cfg = WebConfig::from_env();
    server::run(&cfg);
}

