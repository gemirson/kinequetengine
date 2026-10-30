//! KCE MCP Server — Model Context Protocol implementation (FT-025).
//!
//! Provides a JSON-RPC 2.0 interface via stdin/stdout for AI agents.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead};
use tokio::io::AsyncBufReadExt;

use kce_metrics::sinkhorn::SinkhornMetric;
use kce_retrieval::engine::{Dataset, RetrievalConfig, RetrievalEngine};
use kce_ais::classifier::{AisClassifier, Classification};
use kce_core::traits::DistanceMetric;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin).lines();

    while let Some(line) = reader.next_line().await? {
        let response = handle_request(&line);
        let response_json = serde_json::to_string(&response).unwrap_or_default();
        println!("{}", response_json);
    }

    Ok(())
}

fn handle_request(line: &str) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return error_response(None, -32700, "Parse error"),
    };

    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "KineContextEngine-MCP",
                    "version": "6.0.0"
                }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "kce_retrieve",
                        "description": "Perform hybrid vector search (Cosine + Prime GCD)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query_vector": { "type": "array", "items": { "type": "number" } },
                                "top_k": { "type": "integer", "default": 5 }
                            },
                            "required": ["query_vector"]
                        }
                    },
                    {
                        "name": "kce_sinkhorn",
                        "description": "Compute Sinkhorn optimal transport distance between distributions",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "vector_a": { "type": "array", "items": { "type": "number" } },
                                "vector_b": { "type": "array", "items": { "type": "number" } }
                            },
                            "required": ["vector_a", "vector_b"]
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let tool_name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];

            let result = match tool_name {
                "kce_retrieve" => call_kce_retrieve(args),
                "kce_sinkhorn" => call_kce_sinkhorn(args),
                _ => return error_response(id, -32601, "Method not found"),
            };

            match result {
                Ok(val) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({ "content": [{ "type": "text", "text": val }] })),
                    error: None,
                },
                Err(e) => error_response(id, -32603, &e),
            }
        }
        _ => error_response(id, -32601, "Method not found"),
    }
}

fn error_response(id: Option<Value>, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
    }
}

fn call_kce_retrieve(args: &Value) -> Result<String, String> {
    let query_vector: Vec<f64> = serde_json::from_value(args["query_vector"].clone())
        .map_err(|e| e.to_string())?;
    let top_k = args["top_k"].as_u64().unwrap_or(5) as usize;

    let engine = RetrievalEngine::new(RetrievalConfig::default());
    let mut ds = Dataset::new(query_vector.len());
    // For MCP demo, we use an empty or sample dataset if not provided
    // In production, this would connect to the running KCE server or KineSQL
    ds.push(1, query_vector.clone()).ok(); 

    let results = engine.search(&query_vector, &ds, top_k).map_err(|e| e.to_string())?;
    Ok(serde_json::to_string_pretty(&results).unwrap_or_default())
}

fn call_kce_sinkhorn(args: &Value) -> Result<String, String> {
    let a: Vec<f64> = serde_json::from_value(args["vector_a"].clone()).map_err(|e| e.to_string())?;
    let b: Vec<f64> = serde_json::from_value(args["vector_b"].clone()).map_err(|e| e.to_string())?;

    let metric = SinkhornMetric::with_defaults();
    let dist = metric.compute(&a, &b).map_err(|e| e.to_string())?;
    Ok(format!("{}", dist))
}
