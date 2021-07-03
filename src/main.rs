use anyhow::*;
use clap::Clap;

mod common;
use common::*;
mod members;
mod repositories;

fn main() -> std::result::Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let env: Env = envy::from_env().context("Failed to read necessary environment values")?;
    let config = Config::new();
    let config = config.update_with_opts(opts).update_with_env(env);

    match config.subcmd {
        SubCommand::All(_) => {
            members::process(&config)?;
            repositories::process(&config)?;
            Ok(())
        }
        SubCommand::Members(_) => members::process(&config),
        SubCommand::Repositories(_) => repositories::process(&config),
    }
}
