use serde::{Deserialize, Serialize};
use worker_api::{guest_log, wasm_handler};

#[derive(Serialize, Deserialize, Debug)]
struct AddRequest {
    first: String,
    last: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddResponse {
    full_name: String,
}

fn add(req: AddRequest) -> AddResponse {
    guest_log!("guest: handle() entered");
    guest_log!(
        "guest: parsed AddRequest first={}, last={}",
        req.first,
        req.last
    );

    let res = AddResponse {
        full_name: format!("{} {}", req.first, req.last),
    };

    guest_log!("guest: full_name={}", res.full_name);

    res
}

// Use the macro with AddRequest -> AddResponse
wasm_handler!(add(AddRequest) -> AddResponse);