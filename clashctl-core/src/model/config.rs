use serde::{Deserialize, Serialize};

use super::{deserialize_null_default, Level, Mode};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub port: u64,
    pub socks_port: u64,
    pub redir_port: u64,
    pub tproxy_port: u64,
    pub mixed_port: u64,
    pub allow_lan: bool,
    pub ipv6: bool,
    pub mode: Mode,
    pub log_level: Level,
    pub bind_address: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub authentication: Vec<String>,
}

#[test]
fn null_authentication_is_treated_as_empty() {
    let config: Config = serde_json::from_str(
        r#"{
            "port": 0,
            "socks-port": 0,
            "redir-port": 0,
            "tproxy-port": 0,
            "mixed-port": 0,
            "allow-lan": true,
            "ipv6": true,
            "mode": "rule",
            "log-level": "info",
            "bind-address": "*",
            "authentication": null
        }"#,
    )
    .unwrap();

    assert!(config.authentication.is_empty());
}
