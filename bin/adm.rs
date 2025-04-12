use cdb_adm::{
    boot_out, boot_up, escalate, list_agents_and_daemons, list_agents_and_daemons_paths, turn_off,
    Result, Uid,
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
    BootOut(BootOut),
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
    display_warnings: bool,

    #[arg(short, long)]
    include_non_needed: bool,

    #[arg(short = 'U', long)]
    include_system_uids: bool,
}
#[derive(Args, Debug)]
pub struct BootOut {
    #[arg(short, long, default_value = "501")]
    uid: Option<Uid>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    display_warnings: bool,
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
    display_warnings: bool,

    #[arg(short, long)]
    include_non_needed: bool,

    #[arg(short = 'U', long)]
    include_system_uids: bool,
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
            escalate()?;
            let (success, errors) = turn_off(
                args.uid.clone(),
                !args.verbose,
                !args.display_warnings,
                args.user_services.clone(),
                args.system_services.clone(),
                args.include_non_needed,
                args.include_system_uids,
            );

            if success.len() > 0 {
                println!("{} agents or daemons turned off:\n{}", success.len(), success.join("\n"));
            }
            if errors.len() > 0 {
                println!("{} agents or daemons might be already turned off", errors.len(),);
            }
        },
        Command::BootOut(args) => {
            escalate()?;
            let (success, errors) =
                boot_out(args.uid.clone(), !args.verbose, !args.display_warnings);

            if success.len() > 0 {
                println!("{} agents or daemons booted out", success.len());
            }
            if errors.len() > 0 {
                println!("{} agents or daemons might be already booted out", errors.len(),);
            }
        },
        Command::BootUp(args) => {
            escalate()?;
            let (success, errors) = boot_up(
                args.uid.clone(),
                !args.verbose,
                !args.display_warnings,
                args.user_services.clone(),
                args.system_services.clone(),
                args.include_non_needed,
                args.include_system_uids,
            );

            if success.len() > 0 {
                println!("{} agents or daemons booted up:\n{}", success.len(), success.join("\n"));
            }
            if errors.len() > 0 {
                println!("{} agents or daemons might be already booted up", errors.len(),);
            }
        },
    }
    Ok(())
}
