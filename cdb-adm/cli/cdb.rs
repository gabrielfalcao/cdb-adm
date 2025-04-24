use clap::{Args, Parser, Subcommand};
use iocore::Path;

use crate::cli::{ArgsDispatcher, ParserDispatcher, SubcommandDispatcher};
use crate::{
    coredata_fix, delete_domains, export_domains, export_library_preferences, list_domains, Error,
    Result,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "CDB Command-Line")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
impl ParserDispatcher<Error> for Cli {
    fn dispatch(&self) -> Result<()> {
        self.command.dispatch()?;
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Delete(Delete),
    Fix(Fix),

    Export(Export),
    List(List),
}
impl SubcommandDispatcher<Error> for Command {
    fn dispatch(&self) -> Result<()> {
        match self {
            Command::List(op) => op.dispatch()?,
            Command::Delete(op) => op.dispatch()?,
            Command::Fix(op) => op.dispatch()?,
            Command::Export(op) => op.dispatch()?,
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct Fix {
    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short, long)]
    pub dry_run: bool,
}
impl ArgsDispatcher<Error> for Fix {
    fn dispatch(&self) -> Result<()> {
        coredata_fix(self.quiet, self.dry_run)?;
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct Export {
    #[arg()]
    pub domains: Vec<String>,

    #[arg(short, long)]
    pub no_global: bool,

    #[arg(short, long)]
    pub library_preferences: bool,

    #[arg(short, long)]
    pub output_path: Option<Path>,
}
impl ArgsDispatcher<Error> for Export {
    fn dispatch(&self) -> Result<()> {
        let mut result = export_domains(
            &self
                .domains
                .iter()
                .filter(|domain| !domain.is_empty())
                .map(|domain| domain.as_str())
                .collect::<Vec<&str>>(),
            !self.no_global,
        )?;
        if self.library_preferences {
            result.extend(export_library_preferences()?);
        }

        let data = serde_json::to_string_pretty(&result)?;
        match &self.output_path {
            Some(path) => {
                path.write(data.as_bytes())?;
            },
            None => {
                println!("{}", data);
            },
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct List {}
impl ArgsDispatcher<Error> for List {
    fn dispatch(&self) -> Result<()> {
        for domain in list_domains()? {
            println!("{}", &domain);
        }
        Ok(())
    }
}
#[derive(Args, Debug)]
pub struct Delete {
    #[arg()]
    pub domains: Vec<String>,

    #[arg(short, long)]
    pub output_path: Path,
}
impl ArgsDispatcher<Error> for Delete {
    fn dispatch(&self) -> Result<()> {
        if self.output_path.exists() {
            eprintln!("{} exists", &self.output_path);
            std::process::exit(1);
        }
        let result = delete_domains(
            &self
                .domains
                .iter()
                .filter(|domain| !domain.is_empty())
                .map(|domain| domain.as_str())
                .collect::<Vec<&str>>(),
        )?;
        self.output_path.write(serde_json::to_string_pretty(&result)?.as_bytes())?;
        Ok(())
    }
}
