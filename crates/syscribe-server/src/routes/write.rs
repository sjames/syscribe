use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syscribe_model::frontmatter::split_frontmatter;
use crate::state::SharedState;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PutElementRequest {
    pub name: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WriteOkResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct PositionUpdate {
    pub x: f64,
    pub y: f64,
}

// ---------------------------------------------------------------------------
// PUT /api/elements/{*qname}
// ---------------------------------------------------------------------------

pub async fn put_element(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
    Json(req): Json<PutElementRequest>,
) -> Result<Json<WriteOkResponse>, StatusCode> {
    let qname_norm = qname.replace('/', "::");

    // Find the file path while holding a read lock
    let file_path = {
        let store = state.read().await;
        store
            .elements
            .iter()
            .find(|e| e.qualified_name == qname_norm)
            .map(|e| e.file_path.clone())
            .ok_or(StatusCode::NOT_FOUND)?
    };
    // Read lock is now dropped

    // Read the file from disk
    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        tracing::warn!("Failed to read {:?}: {}", file_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (fm_opt, body) = split_frontmatter(&content);

    // Parse frontmatter YAML into a generic Value (or create empty mapping)
    let mut yaml_val: serde_yaml::Value = match fm_opt {
        Some(yaml_str) => serde_yaml::from_str(yaml_str).unwrap_or(serde_yaml::Value::Mapping(
            serde_yaml::Mapping::new(),
        )),
        None => serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    };

    // Apply name update if provided
    if let Some(ref new_name) = req.name {
        if let serde_yaml::Value::Mapping(ref mut map) = yaml_val {
            map.insert("name".into(), serde_yaml::Value::String(new_name.clone()));
        }
    }

    // Re-serialize YAML
    let new_yaml = serde_yaml::to_string(&yaml_val).map_err(|e| {
        tracing::warn!("Failed to serialize YAML: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Choose body: use req.doc if provided, otherwise the original body
    let final_body = match req.doc.as_deref() {
        Some(d) => d,
        None => body,
    };

    let new_content = format!("---\n{}---\n\n{}", new_yaml, final_body);

    std::fs::write(&file_path, new_content).map_err(|e| {
        tracing::warn!("Failed to write {:?}: {}", file_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(WriteOkResponse { ok: true }))
}

// ---------------------------------------------------------------------------
// PATCH /api/diagrams/{*qname}/layout
// ---------------------------------------------------------------------------

pub async fn patch_layout(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
    Json(positions): Json<HashMap<String, PositionUpdate>>,
) -> Result<Json<WriteOkResponse>, StatusCode> {
    let qname_norm = qname.replace('/', "::");

    // Find the file path while holding a read lock
    let file_path = {
        let store = state.read().await;
        store
            .elements
            .iter()
            .find(|e| e.qualified_name == qname_norm)
            .map(|e| e.file_path.clone())
            .ok_or(StatusCode::NOT_FOUND)?
    };
    // Read lock is now dropped

    // Read the file from disk
    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        tracing::warn!("Failed to read {:?}: {}", file_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (fm_opt, body) = split_frontmatter(&content);

    // Parse frontmatter YAML into a generic Value (or create empty mapping)
    let mut yaml_val: serde_yaml::Value = match fm_opt {
        Some(yaml_str) => serde_yaml::from_str(yaml_str).unwrap_or(serde_yaml::Value::Mapping(
            serde_yaml::Mapping::new(),
        )),
        None => serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
    };

    // Ensure the top-level value is a Mapping
    if let serde_yaml::Value::Mapping(ref mut top_map) = yaml_val {
        // Get or create the `layout` key
        let layout_key = serde_yaml::Value::String("layout".to_string());
        if !top_map.contains_key(&layout_key) {
            top_map.insert(
                layout_key.clone(),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
        }

        if let Some(serde_yaml::Value::Mapping(ref mut layout_map)) =
            top_map.get_mut(&layout_key)
        {
            for (shape_id, pos) in &positions {
                let shape_key = serde_yaml::Value::String(shape_id.clone());
                if !layout_map.contains_key(&shape_key) {
                    layout_map.insert(
                        shape_key.clone(),
                        serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                    );
                }

                if let Some(serde_yaml::Value::Mapping(ref mut shape_map)) =
                    layout_map.get_mut(&shape_key)
                {
                    let x_val = serde_yaml::Value::Number(
                        serde_yaml::Number::from(pos.x.round() as i64),
                    );
                    let y_val = serde_yaml::Value::Number(
                        serde_yaml::Number::from(pos.y.round() as i64),
                    );
                    shape_map.insert("x".into(), x_val);
                    shape_map.insert("y".into(), y_val);
                    // w and h are intentionally left untouched
                }
            }
        }
    }

    // Re-serialize YAML
    let new_yaml = serde_yaml::to_string(&yaml_val).map_err(|e| {
        tracing::warn!("Failed to serialize YAML: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_content = format!("---\n{}---\n\n{}", new_yaml, body);

    std::fs::write(&file_path, new_content).map_err(|e| {
        tracing::warn!("Failed to write {:?}: {}", file_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(WriteOkResponse { ok: true }))
}
