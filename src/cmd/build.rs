use eyre::{anyhow, Result};
use tracing::instrument;

use crate::{
    interface::{SwitcherCommand, SwitcherParser},
    provider,
};

#[instrument]
pub async fn run(args: SwitcherParser) -> Result<()> {
    let SwitcherCommand::Build(build_args) = args.command else {
        return Err(anyhow!("build called with wrong subcommand"));
    };

    if build_args.host.is_none() {
        return Err(anyhow!(
            "Automatic host name discovery is not yet implemented"
        ));
    }

    if build_args.flake.fragment().is_some() {
        return Err(anyhow!("'--flake' is not allowed to contain a fragment"));
    }

    if build_args.all_systems {
        return Err(anyhow!("'--all-systems' is not yet supported"));
    }

    if build_args.only_system {
        return Err(anyhow!("'--only-system' is not yet supported"));
    }

    if build_args.user.is_empty() {
        return Err(anyhow!("at least one '--user' is currently required"));
    }

    let mut flake_ref = build_args.flake.clone();
    flake_ref
        .set_commit_id(provider::retrieve_commit_identifier(&flake_ref.clone().into()).await)?;

    tracing::info!(%flake_ref, "built base flake ref");

    let mut buildables = build_args
        .user
        .iter()
        .map(|user| {
            format!(
                "homeConfigurations.{}@{}.activationPackage",
                user,
                build_args
                    .host
                    .as_ref()
                    .expect("We checked previously that host is known")
            )
        })
        .collect::<Vec<_>>();

    buildables.push(format!(
        "nixosConfigurations.{}.config.system.build.toplevel",
        build_args
            .host
            .expect("We checked previously that host is known")
    ));

    buildables.iter_mut().for_each(|s| {
        let mut fr = flake_ref.clone();
        fr.set_fragment(&s);
        *s = fr.to_string();
    });

    tracing::info!(?buildables, "collected buildables");

    Ok(())
}
