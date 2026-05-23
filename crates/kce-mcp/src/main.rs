//! MCP (Model Context Protocol) server for the KineContext Engine.
//!
//! Implements a minimal JSON-RPC 2.0 server over stdio that exposes KCE
//! operations as MCP tools:
//!
//! - `kce_retrieve` — hybrid vector retrieval
//! - `kce_classify` — self/non-self classification
//! - `kce_sinkhorn` — Sinkhorn optimal transport distance
//! - `kce_regulate` — network regulation (homeostasis)
//!
//! Protocol:
//! ```json
//! {"jsonrpc": "2.0", "method": "tools/list", "id": 1}
//! {"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "...", "arguments": {...}}, "id": 2}
//! ```

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::io::{self, BufRead, Write};

use kce_ais::classifier::SelfNonSelfClassifier;
use kce_ais::regulation::{NetworkRegulator, SystemMetrics};
use kce_core::traits::DistanceMetric;
use kce_metrics::sinkhorn::SinkhornMetric;
use kce_retrieval::engine::{Dataset, RetrievalEngine};
use serde_json::{json, Value};

// ── Tool definitions ─────────────────────────────────────────────────────────

fn tool_definitions() -> Value {
    json!({
        "tools": [
            {
                "name": "kce_retrieve",
                "description": "Hybrid vector retrieval combining cosine and prime (GCD) similarity.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query_vector": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "The query vector."
                        },
                        "dataset_vectors": {
                            "type": "array",
                            "items": {
                                "type": "array",
                                "items": { "type": "number" }
                            },
                            "description": "Dataset vectors to search against."
                        },
                        "top_k": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Number of top results to return.",
                            "default": 5
                        }
                    },
                    "required": ["query_vector", "dataset_vectors"]
                }
            },
            {
                "name": "kce_classify",
                "description": "Self/non-self classification with confidence scoring (negative selection algorithm).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "vector": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "Vector to classify."
                        },
                        "self_patterns": {
                            "type": "array",
                            "items": {
                                "type": "array",
                                "items": { "type": "number" }
                            },
                            "description": "Known self-patterns (at least 3 recommended)."
                        }
                    },
                    "required": ["vector", "self_patterns"]
                }
            },
            {
                "name": "kce_sinkhorn",
                "description": "Sinkhorn entropic-regularized optimal transport distance between two distributions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "a": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "First distribution (non-negative values)."
                        },
                        "b": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "Second distribution (non-negative values, same length as a)."
                        }
                    },
                    "required": ["a", "b"]
                }
            },
            {
                "name": "kce_regulate",
                "description": "Network regulation — evaluates system metrics and returns a regulatory action.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "latency_p95_ms": {
                            "type": "number",
                            "description": "95th percentile query latency in milliseconds."
                        },
                        "error_rate": {
                            "type": "number",
                            "description": "Error rate (0.0 to 1.0)."
                        },
                        "pheromone_entropy": {
                            "type": "number",
                            "description": "Pheromone trail entropy (0.0 = all same, 1.0 = uniform)."
                        },
                        "anomaly_rate": {
                            "type": "number",
                            "description": "Anomalies detected per minute."
                        },
                        "resource_utilization": {
                            "type": "number",
                            "description": "CPU/memory utilization (0.0 to 1.0)."
                        }
                    },
                    "required": ["latency_p95_ms", "error_rate", "pheromone_entropy", "anomaly_rate", "resource_utilization"]
                }
            }
        ]
    })
}

// ── Tool execution ───────────────────────────────────────────────────────────

fn execute_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "kce_retrieve" => {
            let query = args["query_vector"]
                .as_array()
                .ok_or("missing query_vector")?
                .iter()
                .map(|v| v.as_f64().ok_or("query_vector must be numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let dataset_vecs = args["dataset_vectors"]
                .as_array()
                .ok_or("missing dataset_vectors")?;

            let top_k = args["top_k"].as_u64().unwrap_or(5) as usize;
            let dimension = query.len();

            let mut dataset = Dataset::new(dimension);
            for (i, vec_val) in dataset_vecs.iter().enumerate() {
                let v: Vec<f64> = vec_val
                    .as_array()
                    .ok_or("each dataset vector must be an array")?
                    .iter()
                    .map(|x| x.as_f64().ok_or("dataset values must be numbers"))
                    .collect::<Result<Vec<f64>, _>>()?;
                dataset
                    .push(i as u64, v)
                    .map_err(|e| e.to_string())?;
            }

            if dataset.is_empty() {
                return Ok(json!({"results": [], "count": 0}));
            }

            let engine = RetrievalEngine::with_defaults();
            let results = engine
                .search(&query, &dataset, top_k)
                .map_err(|e| e.to_string())?;

            let result_values: Vec<Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "score": r.score,
                        "cosine_score": r.cosine_score,
                        "prime_score": r.prime_score
                    })
                })
                .collect();

            Ok(json!({
                "results": result_values,
                "count": result_values.len()
            }))
        }

        "kce_classify" => {
            let vector: Vec<f64> = args["vector"]
                .as_array()
                .ok_or("missing vector")?
                .iter()
                .map(|v| v.as_f64().ok_or("vector must be numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let patterns = args["self_patterns"]
                .as_array()
                .ok_or("missing self_patterns")?;

            let mut classifier = SelfNonSelfClassifier::default();
            for p in patterns {
                let pattern: Vec<f64> = p
                    .as_array()
                    .ok_or("each self_pattern must be an array")?
                    .iter()
                    .map(|x| x.as_f64().ok_or("pattern values must be numbers"))
                    .collect::<Result<Vec<f64>, _>>()?;
                classifier.register_self(pattern);
            }

            let result = classifier.classify(&vector);
            let label = match result.label {
                kce_ais::classifier::Classification::Self_ => "self",
                kce_ais::classifier::Classification::NonSelf => "non_self",
            };

            Ok(json!({
                "label": label,
                "confidence": result.confidence
            }))
        }

        "kce_sinkhorn" => {
            let a: Vec<f64> = args["a"]
                .as_array()
                .ok_or("missing a")?
                .iter()
                .map(|v| v.as_f64().ok_or("a must be numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let b: Vec<f64> = args["b"]
                .as_array()
                .ok_or("missing b")?
                .iter()
                .map(|v| v.as_f64().ok_or("b must be numbers"))
                .collect::<Result<Vec<f64>, _>>()?;

            let metric = SinkhornMetric::with_defaults();
            let distance = metric.compute(&a, &b).map_err(|e| e.to_string())?;

            Ok(json!({
                "distance": distance,
                "metric": "sinkhorn"
            }))
        }

        "kce_regulate" => {
            let metrics = SystemMetrics {
                latency_p95_ms: args["latency_p95_ms"]
                    .as_f64()
                    .ok_or("missing latency_p95_ms")?,
                error_rate: args["error_rate"]
                    .as_f64()
                    .ok_or("missing error_rate")?,
                pheromone_entropy: args["pheromone_entropy"]
                    .as_f64()
                    .ok_or("missing pheromone_entropy")?,
                anomaly_rate: args["anomaly_rate"]
                    .as_f64()
                    .ok_or("missing anomaly_rate")?,
                resource_utilization: args["resource_utilization"]
                    .as_f64()
                    .ok_or("missing resource_utilization")?,
            };

            let mut regulator = NetworkRegulator::default();
            let adjustments = regulator.regulate_and_apply(metrics);
            let status = regulator.status();

            Ok(json!({
                "action": format!("{:?}", status.last_action),
                "status": status.status,
                "state": {
                    "detection_threshold": status.state.detection_threshold,
                    "amplification_cap": status.state.amplification_cap,
                    "mutation_rate": status.state.mutation_rate,
                    "evaporation_rho": status.state.evaporation_rho,
                    "ant_count": status.state.ant_count,
                    "nonself_threshold": status.state.nonself_threshold,
                    "expiry_days": status.state.expiry_days
                },
                "adjustments": adjustments.iter().map(|a| {
                    json!({
                        "parameter": a.parameter,
                        "from_value": a.from_value,
                        "to_value": a.to_value,
                        "reason": a.reason
                    })
                }).collect::<Vec<Value>>()
            }))
        }

        other => Err(format!("unknown tool: {}", other)),
    }
}

// ── JSON-RPC dispatcher ──────────────────────────────────────────────────────

fn dispatch(request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    match method {
        "initialize" => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "kce-mcp-server",
                        "version": "6.0.0"
                    }
                }
            })
        }

        "tools/list" => {
            let tools = tool_definitions();
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": tools
            })
        }

        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));

            match execute_tool(tool_name, &arguments) {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into())
                            }
                        ]
                    }
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": e
                    }
                }),
            }
        }

        "notifications/initialized" => {
            // Notification — no response needed.
            json!(null)
        }

        _ => {
            if request.get("id").is_some() {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("method not found: {}", method)
                    }
                })
            } else {
                // Notification we don't handle — no response.
                json!(null)
            }
        }
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("parse error: {}", e)
                    }
                });
                let _ = writeln!(stdout, "{}", err_resp);
                let _ = stdout.flush();
                continue;
            }
        };

        let response = dispatch(&request);
        if response.is_null() {
            // Notification — no response to send.
            continue;
        }

        let _ = writeln!(stdout, "{}", response);
        let _ = stdout.flush();
    }
}
