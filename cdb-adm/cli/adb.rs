use std::fmt::Alignment::{Left, Right};

use clap::{Args, Parser, Subcommand};
use verynicetable::Table;

use crate::cli::{ArgsDispatcher, ParserDispatcher, SubcommandDispatcher};
use crate::{
    boot_up_smart, list_agents_and_daemons, list_all_agents_and_daemons, spctl_global_disable,
    turn_off_mdutil, turn_off_smart, Error, Result, Uid,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Agents and Daemons Manager")]
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
    List(List),
    Path(Path),
    TurnOff(TurnOff),
    BootUp(BootUp),
    Status(Status),
}
impl SubcommandDispatcher<Error> for Command {
    fn dispatch(&self) -> Result<()> {
        match self {
            Command::List(op) => op.dispatch()?,
            Command::Path(op) => op.dispatch()?,
            Command::Status(op) => op.dispatch()?,
            Command::TurnOff(op) => op.dispatch()?,
            Command::BootUp(op) => op.dispatch()?,
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct TurnOff {
    #[arg()]
    pub services: Vec<String>,

    #[arg(long, default_value = "501")]
    pub uid: Uid,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub display_warnings: bool,

    #[arg(short, long)]
    pub include_non_needed: bool,

    #[arg(short = 'u', long)]
    pub include_system_uids: bool,

    #[arg(long)]
    pub logs: bool,
}
impl ArgsDispatcher<Error> for TurnOff {
    fn dispatch(&self) -> Result<()> {
        spctl_global_disable()?;
        turn_off_mdutil()?;
        turn_off_smart(
            &self.uid,
            !self.verbose,
            self.services.clone(),
            self.include_non_needed,
            self.logs
        );

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct BootUp {
    #[arg()]
    pub services: Vec<String>,

    #[arg(long, default_value = "501")]
    pub uid: Uid,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short, long)]
    pub display_warnings: bool,

    #[arg(short, long)]
    pub include_non_needed: bool,
}
impl ArgsDispatcher<Error> for BootUp {
    fn dispatch(&self) -> Result<()> {
        boot_up_smart(&self.uid, self.quiet, self.services.clone(), self.include_non_needed);
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct List {
    #[arg(short, long, default_value = "501")]
    pub uid: Option<Uid>,

    #[arg(short, long)]
    pub path: bool,
}
impl ArgsDispatcher<Error> for List {
    fn dispatch(&self) -> Result<()> {
        let data = list_agents_and_daemons()?
            .iter()
            .map(|(label, path)| vec![label.to_string(), path.to_string()])
            .collect::<Vec<Vec<String>>>();
        let table = Table::new()
            .headers(&["SERVICE", "PATH"])
            .alignments(&[Left, Left])
            .data(&data)
            .to_string();
        print!("{table}");
        Ok(())
    }
}
#[derive(Args, Debug)]
pub struct Path {
    #[arg()]
    pub label: String,
}
impl ArgsDispatcher<Error> for Path {
    fn dispatch(&self) -> Result<()> {
        for (label, path) in list_agents_and_daemons()? {
            if label.as_str() == self.label.as_str() {
                println!("{}", path.to_string());
            }
        }
        Ok(())
    }
}
#[derive(Args, Debug)]
pub struct Status {
    #[arg(short, long, help = "list all agents and daemons off and on")]
    pub all: bool,

    #[arg(short = 'p', long = "path")]
    pub include_path: bool,
}
impl ArgsDispatcher<Error> for Status {
    fn dispatch(&self) -> Result<()> {
        let uid = Uid::from(iocore::User::id()?.uid);
        let mut ads = list_all_agents_and_daemons(&uid)?
            .iter()
            .filter(|(_, _, pid, _, _, _)| if self.all { true } else { *pid != 0 })
            .map(|(domain, service, pid, status, _enabled, info)| {
                vec![
                    service.to_string(),
                    pid.to_string(),
                    domain.to_string(),
                    status.map(|h| h.to_string()).unwrap_or_else(|| "-".to_string()),
                    info.clone().map(|(path, _)| path.to_string()).unwrap_or_default(),
                ]
            })
            .collect::<Vec<Vec<String>>>();
        ads.sort_by_key(|service| service[0].to_string());
        if self.include_path {
            let table = Table::new()
                .headers(&["SERVICE", "PID", "DOMAIN", "STATUS", "PATH"])
                .alignments(&[Left, Left, Left, Left, Left])
                .data(&ads)
                .to_string();

            print!("{table}");
        } else {
            let ads = ads
                .iter()
                .map(|ad| (0..4).map(|h| ad[h].to_string()).collect::<Vec<String>>())
                .collect::<Vec<Vec<String>>>();
            let table = Table::new()
                .headers(&["SERVICE", "PID", "DOMAIN", "STATUS"])
                .alignments(&[Left, Left, Right, Left])
                .data(&ads)
                .to_string();

            print!("{table}");
        }

        Ok(())
    }
}
