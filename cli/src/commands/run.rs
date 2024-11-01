use clap::Args;

#[derive(Args)]
pub struct RunCommand {
    #[arg(short, long)]
    config: Option<String>,
}

impl RunCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }  
}