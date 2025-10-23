use core::str;
use std::fmt::Debug;

use eyre::{Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use tokio::process::Command;
use tracing::instrument;

const ENDPOINT: &str = "https://api.github.com/graphql";

type GitObjectID = String;

mod latest_commit;
mod latest_commit_default_branch;

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
        HeaderValue::from_str(&format!("bearer {}", auth.await?))?,
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
async fn get_gh_creds() -> Result<String> {
    let out = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .await
        .wrap_err("running the command")?
        .stdout;

    Ok(str::from_utf8(&out)
        .wrap_err("converting the output to UTF-8")?
        .trim()
        .to_string())
}
