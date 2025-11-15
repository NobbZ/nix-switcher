use std::fmt::Debug;

use eyre::{anyhow, Result};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;
use tracing::instrument;

use crate::provider::github::{
    latest_commit::latest_commit::{
        LatestCommitRepositoryRefTarget::Commit, LatestCommitRepositoryRefTargetOnCommit as TargetOnCommit
    }, GitObjectID, ENDPOINT
};

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/provider/github/get_commit_sha.graphql",
    schema_path = "src/provider/github/schema_gh.graphql",
    response_derives = "Debug"
)]
pub(crate) struct LatestCommit;

#[instrument(skip(client))]
pub(super) async fn get_commit_sha<S1, S2, S3>(
    client: &Client,
    repo: S1,
    owner: S2,
    branch: S3,
) -> Result<String>
where
    S1: AsRef<str> + Debug,
    S2: AsRef<str> + Debug,
    S3: AsRef<str> + Debug,
{
    let variables = latest_commit::Variables {
        repo: repo.as_ref().into(),
        owner: owner.as_ref().into(),
        branch: branch.as_ref().into(),
    };

    let data = post_graphql::<LatestCommit, _>(client, ENDPOINT, variables)
        .await?
        .data;

    let ref_ = data
        .ok_or_else(|| anyhow!("missing in response: data"))?
        .repository
        .ok_or_else(|| anyhow!("missing in response: repository"))?
        .ref_
        .ok_or_else(|| anyhow!("missing in response: ref"))?;

    let id = ref_.id;

    let target = ref_
        .target
        .ok_or_else(|| anyhow!("missing in response: target"))?;

    if let Commit(TargetOnCommit { oid }) = target {
        tracing::debug!(%id, sha1 = oid, "Found commit SHA1");
        Ok(oid)
    } else {
        Err(anyhow!("Not a commit: {:?} for id {}", target, id))
    }
}
