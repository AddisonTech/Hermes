use chrono::Utc;
use opcua::client::prelude::*;
use serde::{Deserialize, Serialize};
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

    pub async fn connect(&self) -> Result<Arc<RwLock<Session>>, Box<dyn std::error::Error>> {
        let mut client = ClientBuilder::new()
            .application_name("Hermes")
            .application_uri("urn:Hermes")
            .trust_server_certs(true)
            .session_retry_limit(3)
            .client()?;

        let (session, event_loop) = client
            .new_session_from_endpoint(
                (
                    self.endpoint.as_str(),
                    SecurityPolicy::None,
                    MessageSecurityMode::None,
                    UserTokenPolicy::anonymous(),
                ),
                IdentityToken::Anonymous,
            )
            .await?;

        tokio::spawn(event_loop.run());

        Ok(session)
    }

    pub async fn read_nodes(
        session: &Arc<RwLock<Session>>,
        node_ids: &[String],
    ) -> Vec<NodeValue> {
        let read_values: Vec<ReadValueId> = node_ids
            .iter()
            .map(|id| ReadValueId {
                node_id: NodeId::from_str(id).unwrap_or(NodeId::null()),
                attribute_id: AttributeId::Value as u32,
                ..Default::default()
            })
            .collect();

        let session = session.read();
        let results = match session
            .read(&read_values, TimestampsToReturn::Both, 0.0)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return node_ids
                    .iter()
                    .map(|id| NodeValue::error(id, &e.to_string()))
                    .collect();
            }
        };

        node_ids
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

                NodeValue {
                    node_id: id.clone(),
                    value,
                    status,
                    timestamp,
                }
            })
            .collect()
    }

    pub async fn browse(
        session: &Arc<RwLock<Session>>,
        node_id: &str,
        depth: u32,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let start = NodeId::from_str(node_id).unwrap_or(ObjectId::ObjectsFolder.into());
        browse_recursive(session, &start, depth, 0).await
    }
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

async fn browse_recursive(
    session: &Arc<RwLock<Session>>,
    node_id: &NodeId,
    max_depth: u32,
    current_depth: u32,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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

    let session = session.read();
    let results = session.browse(&[desc]).await?;

    let mut children = serde_json::Map::new();
    if let Some(refs) = results.into_iter().next().and_then(|r| r.references) {
        for r in refs {
            let name = r.browse_name.name.as_ref().to_string();
            let child_id = r.node_id.node_id.clone();
            let child = Box::pin(browse_recursive(
                session,  // This needs adjustment - session is read-locked
                &child_id,
                max_depth,
                current_depth + 1,
            ));
            // Note: need to drop session lock before recursing
            children.insert(
                name,
                serde_json::json!({ "node_id": child_id.to_string() }),
            );
        }
    }

    Ok(serde_json::Value::Object(children))
}
