use crate::client::OpcClient;
use crate::logger;
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run(
    endpoint: String,
    nodes: Vec<String>,
    interval: f64,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = OpcClient::new(&endpoint);
    println!("{} {}", "logging to:".cyan().bold(), output);

    let mut row_count = 0u64;

    loop {
        let session = match client.connect().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format!("Connection failed: {e} — retrying...").yellow());
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        loop {
            let values = OpcClient::read_nodes(&session, &nodes).await;

            if let Err(e) = logger::write_row(&output, &values) {
                eprintln!("{}", format!("CSV write error: {e}").red());
            } else {
                row_count += 1;
                print!("\r{} {} rows written", "→".green(), row_count);
            }

            sleep(Duration::from_secs_f64(interval)).await;

            if values.iter().any(|v| v.status.contains("Error")) {
                eprintln!("{}", "\nRead error — reconnecting...".yellow());
                break;
            }
        }
    }
}
