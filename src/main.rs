use clap::{Parser, Subcommand};
use colored::Colorize;

mod api;
mod client;
mod commands;
mod display;
mod logger;

#[derive(Parser)]
#[command(
    name = "hermes",
    about = "OPC-UA bridge — poll, serve, log, and browse",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Poll OPC-UA nodes and display live values in the terminal
    Poll {
        /// OPC-UA server endpoint URL
        #[arg(long, default_value = "opc.tcp://localhost:4840")]
        endpoint: String,

        /// Node IDs to read (e.g. "ns=2;s=MyTag")
        #[arg(required = true, num_args = 1..)]
        nodes: Vec<String>,

        /// Poll interval in seconds
        #[arg(long, short, default_value_t = 1.0)]
        interval: f64,
    },

    /// Poll OPC-UA nodes and serve values via REST API
    Serve {
        #[arg(long, default_value = "opc.tcp://localhost:4840")]
        endpoint: String,

        #[arg(required = true, num_args = 1..)]
        nodes: Vec<String>,

        #[arg(long, short, default_value_t = 1.0)]
        interval: f64,

        /// HTTP port for the REST API
        #[arg(long, short, default_value_t = 4000)]
        port: u16,
    },

    /// Poll OPC-UA nodes and log values to CSV
    Log {
        #[arg(long, default_value = "opc.tcp://localhost:4840")]
        endpoint: String,

        #[arg(required = true, num_args = 1..)]
        nodes: Vec<String>,

        #[arg(long, short, default_value_t = 1.0)]
        interval: f64,

        /// Output CSV file path
        #[arg(long, short, default_value = "hermes_log.csv")]
        output: String,
    },

    /// Browse the OPC-UA server namespace
    Browse {
        #[arg(long, default_value = "opc.tcp://localhost:4840")]
        endpoint: String,

        /// Starting node ID (defaults to Objects folder)
        #[arg(long)]
        node: Option<String>,

        /// Maximum browse depth
        #[arg(long, short, default_value_t = 3)]
        depth: u32,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Poll { endpoint, nodes, interval } => {
            commands::poll::run(endpoint, nodes, interval).await
        }
        Command::Serve { endpoint, nodes, interval, port } => {
            commands::serve::run(endpoint, nodes, interval, port).await
        }
        Command::Log { endpoint, nodes, interval, output } => {
            commands::log::run(endpoint, nodes, interval, output).await
        }
        Command::Browse { endpoint, node, depth } => {
            commands::browse::run(endpoint, node, depth).await
        }
    };

    if let Err(e) = result {
        eprintln!("{}", format!("error: {e:#}").red());
        std::process::exit(1);
    }
}
