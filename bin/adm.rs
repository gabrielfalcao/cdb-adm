use std::fmt::Alignment::{Left, Right};

use cdb_adm::{
    boot_up_smart, list_agents_and_daemons, list_all_agents_and_daemons, spctl_global_disable,
    turn_off_mdutil, turn_off_smart, Result, Uid,
};
use clap::{Args, Parser, Subcommand};
use verynicetable::Table;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Agents and Daemons Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    List(List),
    Path(Path),
    TurnOff(TurnOff),
    BootUp(BootUp),
    Status(Status),
}

#[derive(Args, Debug)]
pub struct TurnOff {
    #[arg()]
    services: Vec<String>,

    #[arg(long, default_value = "501")]
    uid: Uid,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    display_warnings: bool,

    #[arg(short, long)]
    include_non_needed: bool,

    #[arg(short = 'u', long)]
    include_system_uids: bool,

    #[arg(long)]
    pub gui: bool,
}
#[derive(Args, Debug)]
pub struct BootUp {
    #[arg()]
    services: Vec<String>,

    #[arg(long, default_value = "501")]
    uid: Uid,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    display_warnings: bool,

    #[arg(short, long)]
    include_non_needed: bool,

    #[arg(long)]
    pub gui: bool,
}

#[derive(Args, Debug)]
pub struct List {
    #[arg(short, long, default_value = "501")]
    pub uid: Option<Uid>,

    #[arg(short, long)]
    pub path: bool,
}
#[derive(Args, Debug)]
pub struct Path {
    #[arg()]
    pub label: String,
}
#[derive(Args, Debug)]
pub struct Status {
    #[arg(short, long, help = "list all agents and daemons off and on")]
    pub all: bool,

    #[arg(short = 'p', long = "path")]
    pub include_path: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::List(op) => {
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
        },
        Command::Path(op) =>
            for (label, path) in list_agents_and_daemons()? {
                if label.as_str() == op.label.as_str() {
                    println!("{}", path.to_string());
                }
            },
        Command::Status(op) => {
            let uid = Uid::from(iocore::User::id()?.uid);
            let mut ads = list_all_agents_and_daemons(&uid)?
                .iter()
                .filter(|(_, _, pid, _, _, _)| if op.all { true } else { *pid != 0 })
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
            if op.include_path {
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
        },
        Command::TurnOff(args) => {
            spctl_global_disable()?;
            turn_off_mdutil()?;
            turn_off_smart(
                &args.uid,
                !args.verbose,
                args.services,
                args.include_non_needed,
                args.include_system_uids,
            );
        },
        Command::BootUp(args) => {
            boot_up_smart(&args.uid, args.quiet, args.services, args.include_non_needed);
        },
    }
    Ok(())
}
