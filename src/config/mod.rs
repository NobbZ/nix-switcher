use figment::{
    providers::{Env, Format, Json, Serialized, Toml, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub owner: String,
    pub repo: String,
    pub branch: Option<String>,
}

impl Config {
    pub fn figment() -> Figment {
        // TODO: Follow XDG base directory specification
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("/home/nmelzer/.config/switcher/config.toml"))
            .merge(Yaml::file("~/.config/switcher/config.yaml"))
            .merge(Yaml::file("~/.config/switcher/config.yml"))
            .merge(Json::file("~/.config/switcher/config.json"))
            .merge(Env::prefixed("SWITCHER_"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // TODO: find a sensible default, maybe username? Or read from `gh`-CLI?
            owner: "".to_string(),
            repo: "nixos-config".to_string(),
            branch: None,
        }
    }
}
