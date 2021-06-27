use anyhow::*;
use clap::Clap;

mod common;
use common::*;
mod members;
mod repositories;

fn main() -> std::result::Result<(), anyhow::Error> {
    let opts = Opts::parse();
    let config: Env = envy::from_env().context("Failed to read necessary environment values")?;
    members::process(&config, &opts)
}
