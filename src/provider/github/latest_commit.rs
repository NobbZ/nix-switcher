use graphql_client::GraphQLQuery;

use super::GitObjectID;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/provider/github/get_commit_sha.graphql",
    schema_path = "src/provider/github/schema_gh.graphql",
    response_derives = "Debug"
)]
pub(crate) struct LatestCommit;

pub(crate) use latest_commit::*;
