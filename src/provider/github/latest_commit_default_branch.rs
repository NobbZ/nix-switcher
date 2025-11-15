use std::fmt::Debug;

use eyre::{eyre, Result};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;

use crate::provider::github::{
    latest_commit_default_branch::latest_commit_default_branch::{
        LatestCommitDefaultBranchRepositoryDefaultBranchRefTarget::Commit, LatestCommitDefaultBranchRepositoryDefaultBranchRefTargetOnCommit as TargetOnCommit
    }, GitObjectID, ENDPOINT
};

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
    let variables = latest_commit_default_branch::Variables {
        repo: repo.as_ref().into(),
        owner: owner.as_ref().into(),
    };

    let data = post_graphql::<LatestCommitDefaultBranch, _>(client, ENDPOINT, variables)
        .await?
        .data
        .ok_or_else(|| eyre!("missing in response: data"))?;

    let default_branch_ref = data
        .repository
        .ok_or_else(|| eyre!("missing in response: repository"))?
        .default_branch_ref
        .ok_or_else(|| eyre!("missing in response: ref"))?;

    let id = default_branch_ref.id;

    let name = default_branch_ref.name;

    let target = default_branch_ref
        .target
        .ok_or_else(|| eyre!("missing in response: target"))?;

    if let Commit(TargetOnCommit { oid }) = target {
        tracing::debug!(%id, branch = name, sha1 = oid, "Found commit SHA1");
        Ok(oid)
    } else {
        Err(eyre!("Not a commit: {:?}", target))
    }
}
