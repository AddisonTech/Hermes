use crate::client::OpcClient;
use colored::Colorize;

pub async fn run(
    endpoint: String,
    node: Option<String>,
    depth: u32,
) -> anyhow::Result<()> {
    let client = OpcClient::new(&endpoint);

    println!("{}", "Connecting...".yellow());
    let session = client.connect().await?;

    let root = node.as_deref().unwrap_or("i=85"); // Objects folder
    println!("{} {}", "Browsing from:".cyan().bold(), root);
    println!("{} {}", "Max depth:    ".dimmed(), depth);
    println!("{}", "─".repeat(64).dimmed());

    let tree = OpcClient::browse(&session, root, depth).await?;
    print_tree(&tree, 0);

    Ok(())
}

fn print_tree(value: &serde_json::Value, depth: usize) {
    let indent = "  ".repeat(depth);
    match value {
        serde_json::Value::Object(map) => {
            for (name, child) in map {
                let node_id = child
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                println!(
                    "{}{} {}",
                    indent,
                    name.cyan(),
                    format!("({node_id})").dimmed()
                );
                print_tree(child, depth + 1);
            }
        }
        _ => {}
    }
}
