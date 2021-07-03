use clap::Clap;
use console::{Style, Term};
use serde::*;

#[derive(Clap)]
#[clap()]
pub(crate) struct Opts {
    #[clap(short, long)]
    pub(crate) cache_file_prefix: Option<String>,
    #[clap(name = "ORGANIZATION")]
    pub(crate) org: String,
    #[clap(name = "OUT_FILE")]
    pub(crate) out_csv_file: String,
    #[clap(subcommand)]
    pub(crate) subcmd: SubCommand,
}

#[derive(Clap)]
pub(crate) enum SubCommand {
    #[clap()]
    Members(Members),
    Repositories(Repositories),
    All(All),
}

#[derive(Clap)]
pub(crate) struct Members {}

#[derive(Clap)]
pub(crate) struct Repositories {}

#[derive(Clap)]
pub(crate) struct All {}

#[derive(Deserialize, Debug)]
pub(crate) struct Env {
    pub(crate) github_access_token: String,
}

pub(crate) struct Config {
    pub(crate) github_access_token: String,
    pub(crate) cache_file_prefix: Option<String>,
    pub(crate) org: String,
    pub(crate) out_csv_file: String,
    pub(crate) subcmd: SubCommand,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            github_access_token: "".to_owned(),
            cache_file_prefix: None,
            org: "".to_owned(),
            out_csv_file: "out.csv".to_owned(),
            subcmd: SubCommand::All(All {}),
        }
    }

    pub(crate) fn update_with_env(self, env: Env) -> Self {
        Self {
            github_access_token: env.github_access_token,
            ..self
        }
    }

    pub(crate) fn update_with_opts(self, opts: Opts) -> Self {
        let config = if let Some(cache_file_prefix) = opts.cache_file_prefix {
            Self {
                cache_file_prefix: Some(cache_file_prefix),
                ..self
            }
        } else {
            self
        };

        Self {
            org: opts.org,
            out_csv_file: opts.out_csv_file,
            subcmd: opts.subcmd,
            ..config
        }
    }
}

pub(crate) enum Progress {
    Downloading,
    Downloaded,
}

pub(crate) fn print_progress(status: Progress) -> std::io::Result<()> {
    let stderr = Term::stderr();
    match status {
        Progress::Downloading => {
            let style = Style::new().yellow().bold();
            let s = style.apply_to("Downloading").to_string();
            stderr.write_line(&s)?;
        }
        Progress::Downloaded => {
            let style = Style::new().green().bold();
            let s = style.apply_to("Downloaded").to_string();
            stderr.clear_last_lines(1)?;
            stderr.write_line(&s)?;
        }
    };

    Ok(())
}
