use crate::client::NodeValue;
use chrono::Local;
use colored::Colorize;

pub fn print_header(endpoint: &str) {
    println!("{}", format!(" Hermes — OPC-UA Bridge ").on_cyan().black().bold());
    println!("{} {}", "endpoint:".dimmed(), endpoint.cyan());
    println!("{} {}", "time:    ".dimmed(), Local::now().format("%H:%M:%S").to_string().cyan());
    println!("{}", "─".repeat(64).dimmed());
    println!(
        "{:<40} {:<14} {}",
        "Node ID".bold(),
        "Value".bold(),
        "Status".bold()
    );
    println!("{}", "─".repeat(64).dimmed());
}

pub fn print_nodes(nodes: &[NodeValue]) {
    for node in nodes {
        let status_colored = if node.status.contains("Good") || node.status == "Good" {
            node.status.green()
        } else if node.status.contains("Bad") {
            node.status.red()
        } else {
            node.status.yellow()
        };

        let value_str = match &node.value {
            serde_json::Value::Null => "—".dimmed().to_string(),
            v => format!("{v}").white().to_string(),
        };

        println!("{:<40} {:<14} {}", node.node_id.cyan(), value_str, status_colored);
    }
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[H");
}
