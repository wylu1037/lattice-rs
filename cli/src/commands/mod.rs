mod init;
mod run;

pub use init::InitCommand;
pub use run::RunCommand;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Init(InitCommand),
    Run(RunCommand),
}