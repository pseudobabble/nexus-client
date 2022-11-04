use std::env::VarError;

use clap::{Parser, Subcommand};

use nexus_client::NexusClient;


#[derive(Subcommand)]
enum Command {
    /// Search Nexus repository
    Search {
        /// The repository to search (eg. 'pypi-internal')
        #[clap(value_parser)]
        repository: String,

        /// The package to find (eg. 'document-store')
        #[clap(value_parser)]
        package: String,

        /// The version of the package to find (eg. '0.2.1')
        #[clap(value_parser)]
        version: Option<String>,
    },
    /// Download Nexus repository artifact
    Download {
        /// The repository to search (eg. 'pypi-internal')
        #[clap(value_parser)]
        repository: String,

        /// The package to download (eg. 'document-store')
        #[clap(value_parser)]
        package: String,

        /// The version of the package to find (eg. '0.2.1')
        #[clap(value_parser)]
        version: String,
    },

}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    let nexus = NexusClient::new();

    match cli.command {
        Command::Search { repository, package, version } => {
            match version {
                Some(version_string) => {
                    let search_items = nexus.search(&repository, Some(package.as_str()), Some(version_string.as_str())).ok();
                    println!("{:#?}", search_items);
                }
                None => {
                    let search_items = nexus.search(&repository, Some(package.as_str()), Some(&"")).ok();
                    println!("{:#?}", search_items);
                }
            }
        }
        Command::Download { repository, package, version } => {
            nexus.download(&repository, Some(package.as_str()), Some(version.as_str()))
        }
    }
}
