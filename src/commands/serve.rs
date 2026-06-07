use crate::api::{self, SharedState};
use crate::client::OpcClient;
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub async fn run(
    endpoint: String,
    nodes: Vec<String>,
    interval: f64,
    port: u16,
) -> anyhow::Result<()> {
    let state: SharedState = Arc::new(RwLock::new(Default::default()));

    let router = api::router(state.clone());
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).await?;
    println!("{} http://{}", "REST API:".cyan().bold(), addr);
    println!("{} {}", "endpoint:".dimmed(), endpoint.cyan());
    println!("{} {:?}", "nodes:   ".dimmed(), nodes);

    let poll_state = state.clone();
    let poll_endpoint = endpoint.clone();
    let poll_nodes = nodes.clone();

    tokio::spawn(async move {
        let client = OpcClient::new(&poll_endpoint);
        loop {
            let session = match client.connect().await {
                Ok(s) => s,
                Err(e) => {
                    api::update_state(&poll_state, vec![], Some(e.to_string())).await;
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            loop {
                let values = OpcClient::read_nodes(&session, &poll_nodes).await;
                let has_error = values.iter().any(|v| v.status.contains("Error"));
                let err = if has_error {
                    Some("One or more read errors".to_string())
                } else {
                    None
                };
                api::update_state(&poll_state, values, err).await;
                sleep(Duration::from_secs_f64(interval)).await;

                if has_error {
                    break;
                }
            }
        }
    });

    axum::serve(listener, router).await?;
    Ok(())
}
