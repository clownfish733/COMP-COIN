use clap::Parser;

#[derive(Parser)]
#[command(name = "node-manager")]
#[command(about = "Node management tool")]
pub struct Cli {
    pub node_path: String,
    
    pub operation: String,

    #[arg(long)]
    pub port: usize,
}