use eyre::{Context, Result};
use tracing::instrument;
use url::Url;

pub(crate) mod github;

#[instrument]
pub async fn retrieve_commit_identifier(url: &Url) -> Result<String> {
    let provider = url.scheme();

    match provider {
        "github" => get_github_sha1(url).await,
        _ => unimplemented!("There is no support for {}", provider),
    }
}

#[instrument]
async fn get_github_sha1(url: &Url) -> Result<String> {
    // TODO: also treat `rev` in the query string properly
    let (owner, repo, branch) = match url.path().split('/').collect::<Vec<_>>()[..] {
        [o, r] => (o.to_string(), r.to_string(), "main".to_string()),
        [o, r, ref b @ ..] => (o.to_string(), r.to_string(), b.join("/")),
        _ => unreachable!(),
    };

    github::get_latest_commit(owner, repo, Some(branch))
        .await
        .wrap_err("retrieving latest commit rev")
}
