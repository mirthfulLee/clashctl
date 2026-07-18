use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::TimeType;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Proxies {
    pub proxies: HashMap<String, Proxy>,
}

impl Proxies {
    pub fn normal(&self) -> impl Iterator<Item = (&String, &Proxy)> {
        self.iter().filter(|(_, x)| x.proxy_type.is_normal())
    }

    pub fn groups(&self) -> impl Iterator<Item = (&String, &Proxy)> {
        self.iter().filter(|(_, x)| x.proxy_type.is_group())
    }

    pub fn selectors(&self) -> impl Iterator<Item = (&String, &Proxy)> {
        self.iter().filter(|(_, x)| x.proxy_type.is_selector())
    }

    pub fn built_ins(&self) -> impl Iterator<Item = (&String, &Proxy)> {
        self.iter().filter(|(_, x)| x.proxy_type.is_built_in())
    }
}

impl Deref for Proxies {
    type Target = HashMap<String, Proxy>;
    fn deref(&self) -> &Self::Target {
        &self.proxies
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Proxy {
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub history: Vec<History>,
    pub udp: Option<bool>,

    // Only present in ProxyGroups
    pub all: Option<Vec<String>>,
    pub now: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct History {
    pub time: TimeType,
    pub delay: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
#[cfg_attr(
    feature = "enum_ext",
    derive(strum::EnumString, strum::Display, strum::EnumVariantNames),
    strum(ascii_case_insensitive)
)]
pub enum ProxyType {
    // Built-In types
    Direct,
    Reject,
    RejectDrop,
    Compatible,
    Pass,
    PassRule,
    Rematch,
    Dns,
    // ProxyGroups
    Selector,
    URLTest,
    Fallback,
    LoadBalance,
    // Proxies
    Shadowsocks,
    Vmess,
    #[serde(alias = "VLESS", alias = "vless")]
    Vless,
    ShadowsocksR,
    Http,
    Snell,
    Trojan,
    Socks5,
    AnyTLS,
    Mieru,
    Sudoku,
    Hysteria,
    Hysteria2,
    Tuic,
    ShadowQuic,
    WireGuard,
    Tailscale,
    Ssh,
    Masque,
    TrustTunnel,
    OpenVPN,
    GostRelay,
    // Relay
    Relay,
    // Unknown
    #[serde(other)]
    Unknown,
}

impl ProxyType {
    pub fn is_selector(&self) -> bool {
        matches!(self, ProxyType::Selector)
    }

    pub fn is_group(&self) -> bool {
        matches!(
            self,
            ProxyType::Selector
                | ProxyType::URLTest
                | ProxyType::Fallback
                | ProxyType::LoadBalance
                | ProxyType::Relay
        )
    }

    pub fn is_built_in(&self) -> bool {
        matches!(
            self,
            ProxyType::Direct
                | ProxyType::Reject
                | ProxyType::RejectDrop
                | ProxyType::Compatible
                | ProxyType::Pass
                | ProxyType::PassRule
                | ProxyType::Rematch
                | ProxyType::Dns
        )
    }

    pub fn is_normal(&self) -> bool {
        matches!(
            self,
            ProxyType::Shadowsocks
                | ProxyType::Vmess
                | ProxyType::Vless
                | ProxyType::ShadowsocksR
                | ProxyType::Http
                | ProxyType::Snell
                | ProxyType::Trojan
                | ProxyType::Socks5
                | ProxyType::AnyTLS
                | ProxyType::Mieru
                | ProxyType::Sudoku
                | ProxyType::Hysteria
                | ProxyType::Hysteria2
                | ProxyType::Tuic
                | ProxyType::ShadowQuic
                | ProxyType::WireGuard
                | ProxyType::Tailscale
                | ProxyType::Ssh
                | ProxyType::Masque
                | ProxyType::TrustTunnel
                | ProxyType::OpenVPN
                | ProxyType::GostRelay
        )
    }
}

#[test]
fn test_supported_proxy_types() {
    let cases = [
        ("Direct", ProxyType::Direct, (true, false, false)),
        ("Reject", ProxyType::Reject, (true, false, false)),
        ("RejectDrop", ProxyType::RejectDrop, (true, false, false)),
        ("Compatible", ProxyType::Compatible, (true, false, false)),
        ("Pass", ProxyType::Pass, (true, false, false)),
        ("PassRule", ProxyType::PassRule, (true, false, false)),
        ("Rematch", ProxyType::Rematch, (true, false, false)),
        ("Dns", ProxyType::Dns, (true, false, false)),
        ("Selector", ProxyType::Selector, (false, true, false)),
        ("URLTest", ProxyType::URLTest, (false, true, false)),
        ("Fallback", ProxyType::Fallback, (false, true, false)),
        ("LoadBalance", ProxyType::LoadBalance, (false, true, false)),
        ("Relay", ProxyType::Relay, (false, true, false)),
        ("Shadowsocks", ProxyType::Shadowsocks, (false, false, true)),
        ("Vmess", ProxyType::Vmess, (false, false, true)),
        ("Vless", ProxyType::Vless, (false, false, true)),
        (
            "ShadowsocksR",
            ProxyType::ShadowsocksR,
            (false, false, true),
        ),
        ("Http", ProxyType::Http, (false, false, true)),
        ("Snell", ProxyType::Snell, (false, false, true)),
        ("Trojan", ProxyType::Trojan, (false, false, true)),
        ("Socks5", ProxyType::Socks5, (false, false, true)),
        ("AnyTLS", ProxyType::AnyTLS, (false, false, true)),
        ("Mieru", ProxyType::Mieru, (false, false, true)),
        ("Sudoku", ProxyType::Sudoku, (false, false, true)),
        ("Hysteria", ProxyType::Hysteria, (false, false, true)),
        ("Hysteria2", ProxyType::Hysteria2, (false, false, true)),
        ("Tuic", ProxyType::Tuic, (false, false, true)),
        ("ShadowQuic", ProxyType::ShadowQuic, (false, false, true)),
        ("WireGuard", ProxyType::WireGuard, (false, false, true)),
        ("Tailscale", ProxyType::Tailscale, (false, false, true)),
        ("Ssh", ProxyType::Ssh, (false, false, true)),
        ("Masque", ProxyType::Masque, (false, false, true)),
        ("TrustTunnel", ProxyType::TrustTunnel, (false, false, true)),
        ("OpenVPN", ProxyType::OpenVPN, (false, false, true)),
        ("GostRelay", ProxyType::GostRelay, (false, false, true)),
    ];

    for (api_name, proxy_type, expected_kind) in cases {
        let json = format!(r#""{api_name}""#);
        let parsed = serde_json::from_str::<ProxyType>(&json).unwrap();

        assert_eq!(parsed, proxy_type, "failed to parse {api_name}");
        assert_eq!(
            serde_json::to_string(&proxy_type).unwrap(),
            json,
            "failed to serialize {api_name}"
        );
        assert_eq!(
            (
                proxy_type.is_built_in(),
                proxy_type.is_group(),
                proxy_type.is_normal()
            ),
            expected_kind,
            "incorrect classification for {api_name}"
        );
    }

    for alias in ["VLESS", "vless"] {
        assert_eq!(
            serde_json::from_str::<ProxyType>(&format!(r#""{alias}""#)).unwrap(),
            ProxyType::Vless
        );
    }
}

#[test]
fn test_proxies() {
    let proxy_kv = [
        (
            "test_a".to_owned(),
            Proxy {
                proxy_type: ProxyType::Direct,
                history: vec![],
                udp: Some(false),
                all: None,
                now: None,
            },
        ),
        (
            "test_b".to_owned(),
            Proxy {
                proxy_type: ProxyType::Selector,
                history: vec![],
                udp: Some(false),
                all: Some(vec!["test_c".into()]),
                now: Some("test_c".into()),
            },
        ),
        (
            "test_c".to_owned(),
            Proxy {
                proxy_type: ProxyType::Shadowsocks,
                history: vec![],
                udp: Some(false),
                all: None,
                now: None,
            },
        ),
        (
            "test_d".to_owned(),
            Proxy {
                proxy_type: ProxyType::Fallback,
                history: vec![],
                udp: Some(false),
                all: Some(vec!["test_c".into()]),
                now: Some("test_c".into()),
            },
        ),
    ];
    let proxies = Proxies {
        proxies: HashMap::from(proxy_kv),
    };
    assert_eq!(
        {
            let mut tmp = proxies.groups().map(|x| x.0).collect::<Vec<_>>();
            tmp.sort();
            tmp
        },
        vec!["test_b", "test_d"]
    );
    assert_eq!(
        proxies.built_ins().map(|x| x.0).collect::<Vec<_>>(),
        vec!["test_a"]
    );
    assert_eq!(
        proxies.normal().map(|x| x.0).collect::<Vec<_>>(),
        vec!["test_c"]
    );
}
