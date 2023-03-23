use std::{collections::HashMap, fmt::Debug};

use anyhow::{anyhow, Result};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;
use tracing::instrument;

const ENDPOINT: &str = "https://api.github.com/graphql";

type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/provider/github/get_commit_sha.graphql",
    schema_path = "src/provider/github/schema_gh.graphql",
    response_derives = "Debug"
)]
pub(crate) struct LatestCommit;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/provider/github/get_commit_sha_default_branch.graphql",
    schema_path = "src/provider/github/schema_gh.graphql",
    response_derives = "Debug"
)]
pub(crate) struct LatestCommitDefaultBranch;

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
    use latest_commit::LatestCommitRepositoryRefTarget::Commit;
    use latest_commit::LatestCommitRepositoryRefTargetOnCommit;
    use latest_commit_default_branch::LatestCommitDefaultBranchRepositoryDefaultBranchRefTarget::Commit as CommitDefaultBranch;
    use latest_commit_default_branch::LatestCommitDefaultBranchRepositoryDefaultBranchRefTargetOnCommit;

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
        let variables = latest_commit::Variables {
            repo: repo.as_ref().into(),
            owner: owner.as_ref().into(),
            branch: branch_name.as_ref().into(),
        };

        let target = post_graphql::<LatestCommit, _>(&client, ENDPOINT, variables)
            .await?
            .data
            .ok_or_else(|| anyhow!("missing in response: data"))?
            .repository
            .ok_or_else(|| anyhow!("missing in response: repository"))?
            .ref_
            .ok_or_else(|| anyhow!("missing in response: ref"))?
            .target
            .ok_or_else(|| anyhow!("missing in response: target"))?;

        if let Commit(LatestCommitRepositoryRefTargetOnCommit { oid }) = target {
            Ok(oid)
        } else {
            Err(anyhow!("Not a commit: {:?}", target))
        }
    } else {
        let variables = latest_commit_default_branch::Variables {
            repo: repo.as_ref().into(),
            owner: owner.as_ref().into(),
        };

        let target = post_graphql::<LatestCommitDefaultBranch, _>(&client, ENDPOINT, variables)
            .await?
            .data
            .ok_or_else(|| anyhow!("missing in response: data"))?
            .repository
            .ok_or_else(|| anyhow!("missing in response: repository"))?
            .default_branch_ref
            .ok_or_else(|| anyhow!("missing in response: ref"))?
            .target
            .ok_or_else(|| anyhow!("missing in response: target"))?;

        if let CommitDefaultBranch(
            LatestCommitDefaultBranchRepositoryDefaultBranchRefTargetOnCommit { oid },
        ) = target
        {
            Ok(oid)
        } else {
            Err(anyhow!("Not a commit: {:?}", target))
        }
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
