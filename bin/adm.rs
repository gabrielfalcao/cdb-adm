use cdb_adm::{
    boot_up, list_agents_and_daemons, list_agents_and_daemons_paths, turn_off, Error, Result, Uid,
};
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Agents and Daemons Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    List(List),
    TurnOff(TurnOff),
    BootUp(BootUp),
}

#[derive(Args, Debug)]
pub struct TurnOff {
    #[arg(short, long, default_value = "501")]
    uid: Option<Uid>,

    #[arg(short = 's', long)]
    user_services: Vec<String>,

    #[arg(short = 'S', long)]
    system_services: Vec<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    include_non_needed: bool,
}

impl TurnOff {
    pub fn turn_off(&self) -> (Vec<String>, Vec<(String, Error)>) {
        turn_off(
            self.uid.clone(),
            !self.verbose,
            self.user_services.clone(),
            self.system_services.clone(),
            self.include_non_needed,
        )
    }
}
#[derive(Args, Debug)]
pub struct BootUp {
    #[arg(short, long, default_value = "501")]
    uid: Option<Uid>,

    #[arg(short = 's', long)]
    user_services: Vec<String>,

    #[arg(short = 'S', long)]
    system_services: Vec<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    include_non_needed: bool,
}

impl BootUp {
    pub fn boot_up(&self) -> (Vec<String>, Vec<(String, Error)>) {
        boot_up(
            self.uid.clone(),
            !self.verbose,
            self.user_services.clone(),
            self.system_services.clone(),
            self.include_non_needed,
        )
    }
}
#[derive(Args, Debug)]
pub struct List {
    #[arg(short, long, default_value = "501")]
    pub uid: Option<Uid>,

    #[arg(long)]
    pub system: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub qualified: bool,

    #[arg(short, long)]
    pub path: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::List(op) =>
            for agent_or_daemon in if op.path {
                list_agents_and_daemons_paths(true, true, op.system)?
                    .iter()
                    .map(|path| path.to_string())
                    .collect::<Vec<String>>()
            } else {
                list_agents_and_daemons(
                    op.uid.clone(),
                    op.qualified,
                    true,
                    true,
                    op.system,
                    !op.verbose,
                )?
            } {
                println!("{}", &agent_or_daemon);
            },
        Command::TurnOff(args) => {
            let (success, errors) = args.turn_off();
            if success.len() > 0 {
                println!("{} agents or daemons turned off:\n{}", success.len(), success.join("\n"));
            }
            if errors.len() > 0 {
                println!(
                    "{} agents or daemons might be already turned off:\n{}",
                    errors.len(),
                    errors.iter().map(|(n, _)| n.to_string()).collect::<Vec<String>>().join("\n")
                );
            }
        },
        Command::BootUp(args) => {
            let (success, errors) = args.boot_up();
            if success.len() > 0 {
                println!("{} agents or daemons turned off:\n{}", success.len(), success.join("\n"));
            }
            if errors.len() > 0 {
                println!(
                    "{} agents or daemons might be already turned off:\n{}",
                    errors.len(),
                    errors.iter().map(|(n, _)| n.to_string()).collect::<Vec<String>>().join("\n")
                );
            }
        },
    }
    Ok(())
}
