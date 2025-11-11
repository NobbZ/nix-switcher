use eyre::{OptionExt, Result};
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
    // TODO: get rid of deprecated function call (its fine for now as we only target Linuxes)
    #[allow(deprecated)]
    pub fn figment() -> Result<Figment> {
        std::env::home_dir()
            .ok_or_eyre("Unable to read home-directory")
            .map(|home_folder| {
                let config_folder = home_folder.join(".config/switcher");

                // TODO: Follow XDG base directory specification
                Figment::from(Serialized::defaults(Config::default()))
                    .merge(Toml::file(config_folder.join("config.toml")))
                    .merge(Yaml::file(config_folder.join("config.yaml")))
                    .merge(Yaml::file(config_folder.join("config.yml")))
                    .merge(Json::file(config_folder.join("config.json")))
                    .merge(Env::prefixed("SWITCHER_"))
            })
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
