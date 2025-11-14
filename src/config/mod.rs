use eyre::Result;
use figment::{
    providers::{Env, Format, Json, Serialized, Toml, Yaml}, Figment
};
use microxdg::XdgApp;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct Config {
    pub repo: RepoConfig,
    pub activators: ActivatorConfig,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RepoConfig {
    pub owner: String,
    pub repo: String,
    pub branch: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ActivatorConfig {
    pub nixos: Option<bool>,
    pub home_manager: bool,
}

impl Config {
    #[allow(deprecated)]
    pub fn figment(xdg: &XdgApp) -> Result<Figment> {
        let mut folders = vec![xdg.app_config()?];
        folders.append(xdg.app_sys_config()?.as_mut());

        let mut figment = Figment::from(Serialized::defaults(Config::default()));

        for folder in folders {
            tracing::info!("Considering {:?} for configuration", &folder);
            figment = figment
                .merge(Toml::file(folder.join("config.toml")))
                .merge(Yaml::file(folder.join("config.yaml")))
                .merge(Yaml::file(folder.join("config.yml")))
                .merge(Json::file(folder.join("config.json")));
        }

        Ok(figment.merge(Env::prefixed("SWITCHER_")))
    }
}

impl Default for RepoConfig {
    fn default() -> Self {
        RepoConfig {
            // TODO: find a sensible default, maybe username? Or read from `gh`-CLI?
            owner: "".to_string(),
            repo: "nixos-config".to_string(),
            branch: None,
        }
    }
}

impl Default for ActivatorConfig {
    fn default() -> Self {
        ActivatorConfig {
            nixos: None,
            home_manager: true,
        }
    }
}
