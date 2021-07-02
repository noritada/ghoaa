use anyhow::*;
use clap::Clap;

mod common;
use common::*;
mod members;
mod repositories;

fn main() -> std::result::Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let env: Env = envy::from_env().context("Failed to read necessary environment values")?;

    match opts.subcmd {
        SubCommand::All(_) => {
            members::process(&env, &opts)?;
            repositories::process(&env, &opts)?;
            Ok(())
        }
        SubCommand::Members(_) => members::process(&env, &opts),
        SubCommand::Repositories(_) => repositories::process(&env, &opts),
    }
}
