use serde::{Deserialize, Serialize};
use worker_api::{guest_log, host_http, wasm_handler};

#[derive(Deserialize, Serialize, Debug)]
struct Request {
    data: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Response {
    data: String,
}

// Define the type you expect back from the host HTTP call
#[derive(Deserialize, Debug)]
struct HttpBinGet {
    origin: String,
}

fn echo(req: Request) -> Response {
    guest_log!("echo: {}", req.data);

    // Typed, ergonomic call:
    let r: HttpBinGet = match host_http!({
        "url": "https://httpbin.org/get",
        "method": "GET"
    }) {
        Ok(v) => v,
        Err(e) => {
            guest_log!("host_http error: {}", e);
            return Response {
                data: format!("Echo (but HTTP failed): {}", e),
            };
        }
    };

    guest_log!("typed url from httpbin: {}", r.origin);

    Response {
        data: format!("Echo: {}", req.data),
    }
}

wasm_handler!(echo(Request) -> Response);