use cdb_adm::{coredata_fix, delete_domains, export_domains, list_domains, Result};
use clap::{Args, Parser, Subcommand};
use iocore::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "CDB Command-Line")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Delete(Delete),
    Fix(Fix),

    Export(Export),
    List(List),
}

#[derive(Args, Debug)]
pub struct Fix {
    #[arg(short, long)]
    quiet: bool,
}
#[derive(Args, Debug)]
pub struct Export {
    #[arg()]
    domains: Vec<String>,

    #[arg(short, long)]
    no_global: bool,

    #[arg(short, long)]
    output_path: Option<Path>,
}
#[derive(Args, Debug)]
pub struct List {}
#[derive(Args, Debug)]
pub struct Delete {
    #[arg()]
    domains: Vec<String>,

    #[arg(short, long)]
    output_path: Path,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::List(_) =>
            for domain in list_domains()? {
                println!("{}", &domain);
            },
        Command::Fix(op) => {
            coredata_fix(op.quiet)?;
        },
        Command::Export(op) => {
            let result = export_domains(
                &op.domains
                    .iter()
                    .filter(|domain| !domain.is_empty())
                    .map(|domain| domain.as_str())
                    .collect::<Vec<&str>>(),
                !op.no_global,
            )?;
            let data = serde_json::to_string_pretty(&result)?;
            match op.output_path {
                Some(path) => {
                    path.write(data.as_bytes())?;
                },
                None => {
                    println!("{}", data);
                },
            }
        },
        Command::Delete(op) => {
            if op.output_path.exists() {
                eprintln!("{} exists", &op.output_path);
                std::process::exit(1);
            }
            let result = delete_domains(
                &op.domains
                    .iter()
                    .filter(|domain| !domain.is_empty())
                    .map(|domain| domain.as_str())
                    .collect::<Vec<&str>>(),
            )?;
            op.output_path.write(serde_json::to_string_pretty(&result)?.as_bytes())?;
        },
    }
    Ok(())
}
