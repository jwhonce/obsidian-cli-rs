mod cli;
mod commands;
mod config;
mod errors;
mod frontmatter;
mod template;
mod types;
mod utils;

mod mcp_server;

use clap::Parser;

use cli::Cli;

fn main() {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("Failed to create async runtime: {e}");
            std::process::exit(1);
        }
    };
    
    let cli = Cli::parse();
    if let Err(e) = rt.block_on(cli.run()) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
