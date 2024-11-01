use clap::Args;

#[derive(Args)]
pub struct InitCommand {
    #[arg(short, long)]
    name: String,
}

impl InitCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }
}