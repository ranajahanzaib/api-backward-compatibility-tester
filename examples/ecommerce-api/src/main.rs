use serde_json::{json, Value};
use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process,
};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Baseline,
    Candidate,
    Compatible,
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args
        .windows(2)
        .find(|p| p[0] == "--mode")
        .and_then(|p| p[1].parse::<String>().ok())
        .unwrap_or_else(|| "baseline".into());
    let mode = match mode.as_str() {
        "baseline" => Mode::Baseline,
        "candidate" => Mode::Candidate,
        "compatible" => Mode::Compatible,
        _ => {
            eprintln!("--mode must be baseline, candidate, or compatible");
            process::exit(1)
        }
    };
    let port = args
        .windows(2)
        .find(|p| p[0] == "--port")
        .and_then(|p| p[1].parse::<u16>().ok())
        .unwrap_or(3001);
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("bind 127.0.0.1:{port}: {e}");
        process::exit(1)
    });
    eprintln!(
        "ecommerce-api {:?} listening on 127.0.0.1:{port}",
        match mode {
            Mode::Baseline => "baseline",
            Mode::Candidate => "candidate",
            Mode::Compatible => "compatible",
        }
    );
    for mut stream in listener.incoming().flatten() {
        let (method, path, headers) = read_request(&mut stream);
        let (status, body) = route(mode, &method, &path);
        respond(&mut stream, status, &body, &headers);
    }
}
fn read_request(stream: &mut TcpStream) -> (String, String, Vec<(String, String)>) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let mut bytes = Vec::new();
    let mut buf = [0; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                bytes.extend_from_slice(&buf[..n]);
                if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
        }
    }
    let text = String::from_utf8_lossy(&bytes);
    let mut lines = text.split("\r\n");
    let first = lines.next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_owned();
    let path = parts.next().unwrap_or("/").to_owned();
    let headers = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| {
            line.split_once(':')
                .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
        })
        .collect();
    (method, path, headers)
}
fn query(path: &str, key: &str) -> Option<String> {
    path.split_once('?')?.1.split('&').find_map(|pair| {
        pair.split_once('=')
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| v.to_owned())
    })
}
fn route(mode: Mode, method: &str, path: &str) -> (u16, Value) {
    if method != "GET" {
        return (405, json!({"error":"method not allowed"}));
    }
    let route = path.split('?').next().unwrap_or(path);
    if route == "/health" {
        return (200, json!({"status":"ok"}));
    }
    if route == "/v2/status" {
        return (200, json!({"status":"ok"}));
    }
    if route == "/customers" {
        if mode == Mode::Candidate && query(path, "region").is_none() {
            return (400, json!({"message":"region is required"}));
        }
        let cursor = query(path, "cursor");
        let next = if cursor.is_none() {
            json!({"next_cursor":"cursor_2"})
        } else {
            json!({})
        };
        let pagination = if mode == Mode::Candidate && cursor.is_none() {
            json!({"next_page":2})
        } else {
            next
        };
        return (
            200,
            json!({"items":[{"id":"cust_123"}],"pagination":pagination}),
        );
    }
    if route.starts_with("/customers/") {
        let id = route.trim_start_matches("/customers/");
        if id != "cust_123" && id != "cust_456" {
            return if mode == Mode::Candidate {
                (400, json!({"message":"Invalid identifier"}))
            } else {
                (
                    422,
                    json!({"error":{"code":"INVALID_CUSTOMER","message":"Invalid customer identifier"}}),
                )
            };
        }
        let value = json!({"customer_id":id,"name":if id=="cust_123"{"Alice Example"}else{"Bob Example"},"status":if mode==Mode::Candidate{"vip"}else{"active"}});
        if mode == Mode::Candidate {
            return (
                200,
                json!({"customer_id":{"value":id},"name":value["name"],"status":value["status"]}),
            );
        }
        if mode == Mode::Compatible {
            return (
                200,
                json!({"customer_id":id,"name":value["name"],"status":"active","loyalty_tier":"standard"}),
            );
        }
        return (200, value);
    }
    if route.starts_with("/orders/") {
        return (
            200,
            json!({"id":route.trim_start_matches("/orders/"),"status":"paid","total":42.50}),
        );
    }
    if route == "/orders" {
        return (
            200,
            json!({"items":[{"id":"ord_100","status":"paid","total":42.50}]}),
        );
    }
    if route.starts_with("/products/") {
        return (
            200,
            json!({"id":route.trim_start_matches("/products/"),"name":"Widget","price":12.0}),
        );
    }
    (
        404,
        json!({"error":{"code":"NOT_FOUND","message":"Not found"}}),
    )
}
fn respond(
    stream: &mut TcpStream,
    status: u16,
    body: &Value,
    request_headers: &[(String, String)],
) {
    let bytes = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    let phrase = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        422 => "Unprocessable Content",
        _ => "Error",
    };
    let _accept_json = request_headers
        .iter()
        .any(|(k, v)| k == "accept" && v.contains("json"));
    let header=format!("HTTP/1.1 {status} {phrase}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&bytes);
}
