//! Stabileo Native & Server Structural Analysis Engine
//!
//! Provides the standalone native execution and server daemon for Dedaliano/Stabileo,
//! ensuring bit-identical results across browser (WASM), desktop, and server workloads
//! (docs/architecture/SOLVER_RUNTIME.md, Issue #486).
//!
//! Usage:
//!   stabileo-engine --solve [FILE]           Solve a model JSON from file or stdin
//!   stabileo-engine --server [--port PORT]   Run HTTP/JSON RPC server for large models
//!   stabileo-engine --sanity [FILE]          Run structural sanity diagnostics
//!   stabileo-engine --build-info             Print engine version and commit metadata
//!   stabileo-engine --version                Print version string

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use dedaliano_engine::solver::linear::{solve_2d, solve_3d};
use dedaliano_engine::solver::{
    solve_buckling_2d, solve_buckling_3d, solve_modal_2d, solve_modal_3d, solve_pdelta_2d,
    solve_pdelta_3d, solve_time_history_2d, solve_time_history_3d,
};
use dedaliano_engine::types::{SolverInput, SolverInput3D, TimeHistoryInput, TimeHistoryInput3D};

/// Determine whether a JSON value represents a 3D structural model.
pub fn is_3d_model(v: &serde_json::Value) -> bool {
    let any_obj = |key: &str, pred: &dyn Fn(&serde_json::Value) -> bool| {
        v.get(key)
            .and_then(|m| m.as_object())
            .map_or(false, |m| m.values().any(pred))
    };
    any_obj("nodes", &|n| n.get("y").is_some() && n.get("z").is_some())
        || any_obj("supports", &|s| s.get("rx").is_some())
}

/// Strip non-deterministic wall-clock timing fields.
pub fn strip_run_metadata(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(m) => {
            m.remove("timings");
            m.remove("solveTimings");
            m.remove("runMeta");
            for val in m.values_mut() {
                strip_run_metadata(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                strip_run_metadata(val);
            }
        }
        _ => {}
    }
}

/// Solve model JSON string and return serialized JSON result.
pub fn execute_solve(json_str: &str, force_3d: Option<bool>) -> Result<String, String> {
    let mut root: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON input: {e}"))?;

    // Check if wrapped in an analysis container: {"analysis": "...", "model": ...}
    if let Some(kind_val) = root.get("analysis").and_then(|k| k.as_str()) {
        let kind = kind_val.to_string();
        let num_modes = root.get("numModes").and_then(|x| x.as_u64()).unwrap_or(4) as usize;
        let max_iter = root.get("maxIter").and_then(|x| x.as_u64()).unwrap_or(100) as usize;
        let tolerance = root.get("tolerance").and_then(|x| x.as_f64()).unwrap_or(1e-10);

        if kind == "timeHistory" {
            let input = root["input"].take();
            let is_3d = force_3d.unwrap_or_else(|| is_3d_model(&input["solver"]));
            return if is_3d {
                let th_in: TimeHistoryInput3D = serde_json::from_value(input)
                    .map_err(|e| format!("Failed to parse TimeHistoryInput3D: {e}"))?;
                let res = solve_time_history_3d(&th_in)?;
                serde_json::to_string(&res).map_err(|e| e.to_string())
            } else {
                let th_in: TimeHistoryInput = serde_json::from_value(input)
                    .map_err(|e| format!("Failed to parse TimeHistoryInput: {e}"))?;
                let res = solve_time_history_2d(&th_in)?;
                serde_json::to_string(&res).map_err(|e| e.to_string())
            };
        }

        let densities: serde_json::Value = root.get_mut("densities").map(|d| d.take()).unwrap_or(serde_json::json!({}));
        let model = root["model"].take();
        let is_3d = force_3d.unwrap_or_else(|| is_3d_model(&model));

        if is_3d {
            let m: SolverInput3D = serde_json::from_value(model)
                .map_err(|e| format!("Failed to parse SolverInput3D: {e}"))?;
            match kind.as_str() {
                "buckling" => {
                    let r = solve_buckling_3d(&m, num_modes)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "modal" => {
                    let dens = serde_json::from_value(densities).unwrap_or_default();
                    let r = solve_modal_3d(&m, &dens, num_modes)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "pdelta" => {
                    let r = solve_pdelta_3d(&m, max_iter, tolerance)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "linear" => {
                    let r = solve_3d(&m)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                other => Err(format!("Unsupported 3D analysis kind: {other}")),
            }
        } else {
            let m: SolverInput = serde_json::from_value(model)
                .map_err(|e| format!("Failed to parse SolverInput: {e}"))?;
            match kind.as_str() {
                "buckling" => {
                    let r = solve_buckling_2d(&m, num_modes)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "modal" => {
                    let dens = serde_json::from_value(densities).unwrap_or_default();
                    let r = solve_modal_2d(&m, &dens, num_modes)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "pdelta" => {
                    let r = solve_pdelta_2d(&m, max_iter, tolerance)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                "linear" => {
                    let r = solve_2d(&m)?;
                    serde_json::to_string(&r).map_err(|e| e.to_string())
                }
                other => Err(format!("Unsupported 2D analysis kind: {other}")),
            }
        }
    } else {
        // Direct model payload
        let is_3d = force_3d.unwrap_or_else(|| is_3d_model(&root));
        if is_3d {
            let m: SolverInput3D = serde_json::from_value(root)
                .map_err(|e| format!("Failed to parse SolverInput3D: {e}"))?;
            let r = solve_3d(&m)?;
            serde_json::to_string(&r).map_err(|e| e.to_string())
        } else {
            let m: SolverInput = serde_json::from_value(root)
                .map_err(|e| format!("Failed to parse SolverInput: {e}"))?;
            let r = solve_2d(&m)?;
            serde_json::to_string(&r).map_err(|e| e.to_string())
        }
    }
}

/// Handle a single incoming HTTP request on the server socket.
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 8192];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    if method == "GET" && (path == "/health" || path == "/") {
        let body = serde_json::json!({
            "status": "ok",
            "service": "stabileo-engine",
            "version": env!("CARGO_PKG_VERSION"),
            "commit": env!("STABILEO_ENGINE_COMMIT")
        })
        .to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if method == "GET" && path == "/build-info" {
        let body = dedaliano_engine::engine_build_info();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if method == "POST" && path == "/solve" {
        // Locate start of HTTP body
        if let Some(pos) = request_str.find("\r\n\r\n") {
            let body = &request_str[pos + 4..];
            match execute_solve(body, None) {
                Ok(result_json) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        result_json.len(),
                        result_json
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(err_msg) => {
                    let err_json = serde_json::json!({ "error": err_msg }).to_string();
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        err_json.len(),
                        err_json
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
        return;
    }

    // 404 fallback
    let not_found = serde_json::json!({ "error": "Not Found" }).to_string();
    let response = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        not_found.len(),
        not_found
    );
    let _ = stream.write_all(response.as_bytes());
}

/// Run daemon HTTP server on the specified port.
pub fn run_server(port: u16) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("Stabileo native/server engine listening on http://{addr}");
    println!("Endpoints: GET /health, GET /build-info, POST /solve");
    for stream in listener.incoming() {
        if let Ok(s) = stream {
            handle_client(s);
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Stabileo Structural Analysis Engine v{}", env!("CARGO_PKG_VERSION"));
        println!("Deterministic native & server runner for large-scale structural models.\n");
        println!("Usage:");
        println!("  stabileo-engine --solve [FILE]           Solve model JSON from file or stdin");
        println!("  stabileo-engine --server [--port PORT]   Start HTTP/JSON RPC server");
        println!("  stabileo-engine --sanity [FILE]          Run structural sanity diagnostics");
        println!("  stabileo-engine --build-info             Print engine version and commit");
        println!("  stabileo-engine --version                Print version");
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("stabileo-engine {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.iter().any(|a| a == "--build-info") {
        println!("{}", dedaliano_engine::engine_build_info());
        return;
    }

    if args.iter().any(|a| a == "--server") {
        let mut port = 8088;
        if let Some(pos) = args.iter().position(|a| a == "--port") {
            if pos + 1 < args.len() {
                port = args[pos + 1].parse().unwrap_or(8088);
            }
        }
        if let Err(e) = run_server(port) {
            eprintln!("Failed to start server: {e}");
            std::process::exit(1);
        }
        return;
    }

    // Default solve execution mode: --solve or positional file or stdin
    let mut file_path: Option<String> = None;
    let mut force_3d: Option<bool> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--solve" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    file_path = Some(args[i].clone());
                }
            }
            "--2d" => force_3d = Some(false),
            "--3d" => force_3d = Some(true),
            other if !other.starts_with('-') && file_path.is_none() => {
                file_path = Some(other.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    let json_content = if let Some(path) = file_path {
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read file '{path}': {e}");
            std::process::exit(1);
        })
    } else {
        let mut stdin_buf = String::new();
        std::io::stdin().read_to_string(&mut stdin_buf).unwrap_or_else(|e| {
            eprintln!("Failed to read from stdin: {e}");
            std::process::exit(1);
        });
        stdin_buf
    };

    match execute_solve(&json_content, force_3d) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            eprintln!("{}", serde_json::json!({ "error": e }));
            std::process::exit(2);
        }
    }
}
