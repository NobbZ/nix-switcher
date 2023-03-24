use std::{collections::HashMap, fmt::Debug};

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;
use tracing::instrument;

const ENDPOINT: &str = "https://api.github.com/graphql";

type GitObjectID = String;

pub(self) mod latest_commit;
pub(self) mod latest_commit_default_branch;

#[derive(Deserialize, Clone)]
struct GhHost {
    // user: String,
    oauth_token: String,
    // git_protocoll: String,
}

#[instrument]
pub(crate) async fn get_latest_commit<S1, S2, S3>(
    owner: S1,
    repo: S2,
    branch: Option<S3>,
) -> Result<String>
where
    S1: AsRef<str> + Debug,
    S2: AsRef<str> + Debug,
    S3: AsRef<str> + Debug,
{
    let auth = get_gh_creds();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("bearer {}", auth.await?.oauth_token))?,
    );

    let client = Client::builder()
        .user_agent("nobbz switcher/0.0")
        .default_headers(headers)
        .build()?;

    if let Some(branch_name) = branch {
        latest_commit::get_commit_sha(&client, repo, owner, branch_name).await
    } else {
        latest_commit_default_branch::get_commit_sha(&client, repo, owner).await
    }
}

#[instrument]
async fn get_gh_creds() -> Result<GhHost> {
    let home = std::env::var("HOME")?;
    let path = std::path::Path::new(&home).join(".config/gh/hosts.yml");
    let f = tokio::fs::File::open(path).await?;
    let d: HashMap<String, GhHost> = serde_yaml::from_reader(f.into_std().await)?;

    d.get("github.com")
        .cloned()
        .ok_or_else(|| anyhow!("Host not configured: github.com"))
}
