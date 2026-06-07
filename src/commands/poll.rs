use crate::client::OpcClient;
use crate::display;
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run(
    endpoint: String,
    nodes: Vec<String>,
    interval: f64,
) -> anyhow::Result<()> {
    let client = OpcClient::new(&endpoint);

    loop {
        println!("{}", "Connecting to OPC-UA server...".yellow());
        let session = match client.connect().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format!("Connection failed: {e}").red());
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        loop {
            let values = OpcClient::read_nodes(&session, &nodes).await;

            display::clear_screen();
            display::print_header(&endpoint);
            display::print_nodes(&values);

            sleep(Duration::from_secs_f64(interval)).await;

            if values.iter().any(|v| v.status.contains("Error")) {
                eprintln!("{}", "Read error — reconnecting...".yellow());
                break;
            }
        }
    }
}
