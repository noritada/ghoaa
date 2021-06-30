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
}

#[derive(Clap)]
pub(crate) struct Members {}

#[derive(Clap)]
pub(crate) struct Repositories {}

#[derive(Deserialize, Debug)]
pub(crate) struct Env {
    pub(crate) github_access_token: String,
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
