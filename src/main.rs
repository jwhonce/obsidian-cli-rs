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
        let exit_code = match &e {
            crate::errors::ObsidianError::FileNotFound { .. } => 2,
            crate::errors::ObsidianError::FileExists { .. } => 3,
            crate::errors::ObsidianError::FrontmatterKeyNotFound { .. } => 4,
            crate::errors::ObsidianError::FrontmatterKeyExists { .. } => 5,
            crate::errors::ObsidianError::InvalidArguments { .. } => 6,
            _ => 1,
        };
        std::process::exit(exit_code);
    }
}
