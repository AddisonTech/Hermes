use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use opcua::client::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeValue {
    pub node_id: String,
    pub value: serde_json::Value,
    pub status: String,
    pub timestamp: String,
}

impl NodeValue {
    pub fn error(node_id: &str, message: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            value: serde_json::Value::Null,
            status: format!("Error: {message}"),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

pub struct OpcClient {
    endpoint: String,
}

impl OpcClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self { endpoint: endpoint.into() }
    }

    pub async fn connect(&self) -> Result<Arc<RwLock<Session>>> {
        let endpoint = self.endpoint.clone();
        tokio::task::spawn_blocking(move || {
            let mut client = ClientBuilder::new()
                .application_name("Hermes")
                .application_uri("urn:Hermes")
                .trust_server_certs(true)
                .session_retry_limit(3)
                .client()
                .ok_or_else(|| anyhow!("Failed to build OPC-UA client"))?;

            let session = client
                .connect_to_endpoint(
                    (
                        endpoint.as_str(),
                        SecurityPolicy::None.to_str(),
                        MessageSecurityMode::None,
                        UserTokenPolicy::anonymous(),
                    ),
                    IdentityToken::Anonymous,
                )
                .map_err(|e| anyhow!("Connect failed: {e:?}"))?;

            Ok(session)
        })
        .await
        .context("spawn_blocking panicked")?
    }

    pub async fn read_nodes(
        session: &Arc<RwLock<Session>>,
        node_ids: &[String],
    ) -> Vec<NodeValue> {
        let read_values: Vec<ReadValueId> = node_ids
            .iter()
            .map(|id| {
                let node_id = NodeId::from_str(id).unwrap_or_else(|_| NodeId::null());
                ReadValueId {
                    node_id,
                    attribute_id: AttributeId::Value as u32,
                    index_range: UAString::null(),
                    data_encoding: QualifiedName::null(),
                }
            })
            .collect();

        let session = session.clone();
        let node_ids_owned = node_ids.to_vec();

        tokio::task::spawn_blocking(move || {
            let guard = session.read();
            match guard.read(&read_values, TimestampsToReturn::Both, 0.0) {
                Ok(results) => node_ids_owned
                    .iter()
                    .zip(results.iter())
                    .map(|(id, dv)| {
                        let status = dv
                            .status
                            .map(|s| format!("{s:?}"))
                            .unwrap_or_else(|| "Unknown".to_string());

                        let value = dv
                            .value
                            .as_ref()
                            .map(variant_to_json)
                            .unwrap_or(serde_json::Value::Null);

                        let timestamp = dv
                            .source_timestamp
                            .map(|t| t.as_chrono().to_rfc3339())
                            .unwrap_or_else(|| Utc::now().to_rfc3339());

                        NodeValue { node_id: id.clone(), value, status, timestamp }
                    })
                    .collect(),
                Err(e) => node_ids_owned
                    .iter()
                    .map(|id| NodeValue::error(id, &format!("{e:?}")))
                    .collect(),
            }
        })
        .await
        .unwrap_or_else(|e| {
            node_ids
                .iter()
                .map(|id| NodeValue::error(id, &e.to_string()))
                .collect()
        })
    }

    pub async fn browse(
        session: &Arc<RwLock<Session>>,
        node_id: &str,
        depth: u32,
    ) -> Result<serde_json::Value> {
        let start = NodeId::from_str(node_id).unwrap_or_else(|_| NodeId::objects_folder_id());
        let session = session.clone();

        tokio::task::spawn_blocking(move || browse_recursive(&session, &start, depth, 0))
            .await
            .context("spawn_blocking panicked")?
    }
}

fn browse_recursive(
    session: &Arc<RwLock<Session>>,
    node_id: &NodeId,
    max_depth: u32,
    current_depth: u32,
) -> Result<serde_json::Value> {
    if current_depth >= max_depth {
        return Ok(serde_json::Value::Null);
    }

    let desc = BrowseDescription {
        node_id: node_id.clone(),
        browse_direction: BrowseDirection::Forward,
        reference_type_id: ReferenceTypeId::HierarchicalReferences.into(),
        include_subtypes: true,
        node_class_mask: 0,
        result_mask: 63,
    };

    let browse_results: Vec<BrowseResult> = {
        let guard = session.read();
        guard
            .browse(&[desc])
            .map_err(|e| anyhow!("Browse failed: {e:?}"))?
            .unwrap_or_default()
    };

    let mut children = serde_json::Map::new();

    for result in browse_results {
        if let Some(refs) = result.references {
            for r in refs {
                let name = r.browse_name.name.as_ref().to_string();
                let child_id = r.node_id.node_id.clone();
                let child =
                    browse_recursive(session, &child_id, max_depth, current_depth + 1)?;
                children.insert(name, child);
            }
        }
    }

    Ok(serde_json::Value::Object(children))
}

fn variant_to_json(v: &Variant) -> serde_json::Value {
    match v {
        Variant::Boolean(b) => serde_json::json!(b),
        Variant::SByte(n) => serde_json::json!(n),
        Variant::Byte(n) => serde_json::json!(n),
        Variant::Int16(n) => serde_json::json!(n),
        Variant::UInt16(n) => serde_json::json!(n),
        Variant::Int32(n) => serde_json::json!(n),
        Variant::UInt32(n) => serde_json::json!(n),
        Variant::Int64(n) => serde_json::json!(n),
        Variant::UInt64(n) => serde_json::json!(n),
        Variant::Float(f) => serde_json::json!(f),
        Variant::Double(f) => serde_json::json!(f),
        Variant::String(s) => serde_json::json!(s.as_ref()),
        _ => serde_json::json!(format!("{v:?}")),
    }
}
