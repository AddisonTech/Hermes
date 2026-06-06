use crate::client::NodeValue;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiState {
    pub nodes: HashMap<String, NodeValue>,
    pub last_updated: Option<String>,
    pub error: Option<String>,
}

impl Default for ApiState {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            last_updated: None,
            error: None,
        }
    }
}

pub type SharedState = Arc<RwLock<ApiState>>;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/nodes", get(get_all_nodes))
        .route("/nodes/:node_id", get(get_node))
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_all_nodes(State(state): State<SharedState>) -> Json<ApiState> {
    Json(state.read().await.clone())
}

async fn get_node(
    Path(node_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<NodeValue>, StatusCode> {
    let state = state.read().await;
    state
        .nodes
        .get(&node_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "time": Utc::now().to_rfc3339(),
    }))
}

pub async fn update_state(state: &SharedState, values: Vec<NodeValue>, error: Option<String>) {
    let mut s = state.write().await;
    s.last_updated = Some(Utc::now().to_rfc3339());
    s.error = error;
    for v in values {
        s.nodes.insert(v.node_id.clone(), v);
    }
}
