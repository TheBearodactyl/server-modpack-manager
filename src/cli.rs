use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(short = 'l', long)]
    pub list_tags: Option<bool>,
    
    #[arg(short = 'r', long)]
    pub repo_info: Option<bool>
}