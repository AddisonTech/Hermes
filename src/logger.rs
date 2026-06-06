use crate::client::NodeValue;
use chrono::Utc;
use std::path::Path;

pub fn write_row(output: &str, values: &[NodeValue]) -> Result<(), Box<dyn std::error::Error>> {
    let file_exists = Path::new(output).exists();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output)?;

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    if !file_exists {
        let mut header = vec!["timestamp".to_string()];
        header.extend(values.iter().map(|v| v.node_id.clone()));
        writer.write_record(&header)?;
    }

    let mut row = vec![Utc::now().to_rfc3339()];
    row.extend(values.iter().map(|v| v.value.to_string()));
    writer.write_record(&row)?;
    writer.flush()?;

    Ok(())
}
