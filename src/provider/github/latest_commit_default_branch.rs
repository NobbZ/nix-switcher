use std::fmt::Debug;

use anyhow::{anyhow, Result};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;

use super::GitObjectID;
use super::ENDPOINT;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/provider/github/get_commit_sha_default_branch.graphql",
    schema_path = "src/provider/github/schema_gh.graphql",
    response_derives = "Debug"
)]
pub(crate) struct LatestCommitDefaultBranch;

pub(super) async fn get_commit_sha<S1, S2>(client: &Client, repo: S1, owner: S2) -> Result<String>
where
    S1: AsRef<str> + Debug,
    S2: AsRef<str> + Debug,
{
    use latest_commit_default_branch::LatestCommitDefaultBranchRepositoryDefaultBranchRefTarget::Commit;
    use latest_commit_default_branch::LatestCommitDefaultBranchRepositoryDefaultBranchRefTargetOnCommit as TargetOnCommit;

    let variables = latest_commit_default_branch::Variables {
        repo: repo.as_ref().into(),
        owner: owner.as_ref().into(),
    };

    let data = post_graphql::<LatestCommitDefaultBranch, _>(client, ENDPOINT, variables)
        .await?
        .data
        .ok_or_else(|| anyhow!("missing in response: data"))?;

    let default_branch_ref = data
        .repository
        .ok_or_else(|| anyhow!("missing in response: repository"))?
        .default_branch_ref
        .ok_or_else(|| anyhow!("missing in response: ref"))?;

    let id = default_branch_ref.id;

    let name = default_branch_ref.name;

    let target = default_branch_ref
        .target
        .ok_or_else(|| anyhow!("missing in response: target"))?;

    if let Commit(TargetOnCommit { oid }) = target {
        tracing::debug!(%id, branch = name, sha1 = oid, "Found commit SHA1");
        Ok(oid)
    } else {
        Err(anyhow!("Not a commit: {:?}", target))
    }
}
