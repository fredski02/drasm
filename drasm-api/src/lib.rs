use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub type_name: String, // e.g. "AddRequest"
    pub payload: Vec<u8>,  // serialized inner request/response (we use JSON bytes)
}

impl Message {
    /// Create a Message whose payload is JSON of `val`.
    pub fn from<T: serde::Serialize>(val: &T) -> Self {
        Message {
            type_name: std::any::type_name::<T>().to_string(),
            payload: serde_json::to_vec(val).expect("payload json serialize"),
        }
    }

    /// Convert the payload into JSON Value by deserializing it as `T`.
    pub fn to_json<T>(&self) -> serde_json::Value
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        match serde_json::from_slice::<T>(&self.payload) {
            Ok(val) => serde_json::to_value(val)
                .unwrap_or(serde_json::Value::String("<json conversion error>".into())),
            Err(_) => serde_json::Value::String("<decode error>".into()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WasmSlice {
    pub ptr: i32,
    pub len: i32,
}

impl WasmSlice {
    #[inline]
    pub fn pack(self) -> i64 {
        ((self.len as u64) << 32 | (self.ptr as u32 as u64)) as i64
    }

    #[inline]
    pub fn unpack(val: i64) -> Self {
        let u = val as u64;
        WasmSlice {
            ptr: (u & 0xffff_ffff) as i32,
            len: (u >> 32) as i32,
        }
    }
}
#[macro_export]
macro_rules! wasm_handler {
    ($func:ident($req:ty) -> $res:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn alloc(len: usize) -> *mut u8 {
            let mut buf = Vec::<u8>::with_capacity(len);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        #[no_mangle]
        pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }

        #[no_mangle]
        pub unsafe extern "C" fn handle(ptr: *const u8, len: usize) -> i64 {
            // read input bytes (envelope)
            let input_slice = std::slice::from_raw_parts(ptr, len);

            // Try to decode Message envelope safely (no panic) using JSON
            let msg_res: Result<$crate::Message, _> = serde_json::from_slice(input_slice);

            // If decoding the envelope failed, build an Error Message and return it
            let msg = match msg_res {
                Ok(m) => m,
                Err(e) => {
                    let err_payload = match serde_json::to_vec(&serde_json::json!({
                        "error": format!("bad envelope: {}", e)
                    })) {
                        Ok(p) => p,
                        Err(_) => b"{}".to_vec(),
                    };

                    let out_msg = $crate::Message {
                        type_name: "Error".to_string(),
                        payload: err_payload,
                    };

                    // Serialize out_msg as JSON
                    let out_bytes = serde_json::to_vec(&out_msg).unwrap_or_else(|_| b"{}".to_vec());
                    let out_len = out_bytes.len();
                    let out_ptr = alloc(out_len);

                    std::ptr::copy_nonoverlapping(out_bytes.as_ptr(), out_ptr, out_len);
                    std::mem::forget(out_bytes);

                    return $crate::WasmSlice {
                        ptr: out_ptr as i32,
                        len: out_len as i32,
                    }
                    .pack();
                }
            };

            // Now try to decode the payload into the concrete request type using JSON
            let req_res: Result<$req, _> = serde_json::from_slice(&msg.payload);

            let req: $req = match req_res {
                Ok(r) => r,
                Err(e) => {
                    // Create Error message describing the bad payload
                    let err_payload = match serde_json::to_vec(&serde_json::json!({
                        "error": format!("bad payload: {}", e)
                    })) {
                        Ok(p) => p,
                        Err(_) => b"{}".to_vec(),
                    };

                    let out_msg = $crate::Message {
                        type_name: "Error".to_string(),
                        payload: err_payload,
                    };

                    let out_bytes = serde_json::to_vec(&out_msg).unwrap_or_else(|_| b"{}".to_vec());
                    let out_len = out_bytes.len();
                    let out_ptr = alloc(out_len);

                    std::ptr::copy_nonoverlapping(out_bytes.as_ptr(), out_ptr, out_len);
                    std::mem::forget(out_bytes);

                    return $crate::WasmSlice {
                        ptr: out_ptr as i32,
                        len: out_len as i32,
                    }
                    .pack();
                }
            };

            // Call user function (now safe: req is present and parsed)
            let res: $res = $func(req);

            // Serialize response to JSON bytes
            let out_payload = serde_json::to_vec(&res).expect("json serialize failed");

            // Wrap into Message
            let out_msg = $crate::Message {
                type_name: stringify!($res).to_string(),
                payload: out_payload,
            };

            // serialize the message as JSON
            let out_bytes = serde_json::to_vec(&out_msg).expect("serialize Message failed");
            let out_len = out_bytes.len();
            let out_ptr = alloc(out_len);

            std::ptr::copy_nonoverlapping(out_bytes.as_ptr(), out_ptr, out_len);
            std::mem::forget(out_bytes);

            $crate::WasmSlice {
                ptr: out_ptr as i32,
                len: out_len as i32,
            }
            .pack()
        }
    };
}

// --- Guest logging helpers ---------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Host-provided logging import. Exact signature: (i32, i32) -> ()
    /// `ptr` is an offset in the guest's linear memory to UTF-8 bytes; `len` is the length.
    fn log(ptr: i32, len: i32);
    /// These are exported by the guest (via `wasm_handler!`).
    pub fn http_request(ptr: i32, len: i32) -> i64;

}

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn alloc(len: usize) -> *mut u8;
    pub fn dealloc(ptr: *mut u8, len: usize);
}

#[cfg(target_arch = "wasm32")]
#[inline]
pub fn host_log_bytes(bytes: &[u8]) {
    unsafe {
        log(bytes.as_ptr() as i32, bytes.len() as i32);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn host_log_bytes(bytes: &[u8]) {
    // Fallback for native/testing: just print to stderr
    eprintln!("[guest log] {}", String::from_utf8_lossy(bytes));
}

#[inline]
pub fn host_log(s: &str) {
    // Optional: chunk long logs to avoid sending a huge buffer in one go.
    const CHUNK: usize = 4096;
    if s.len() <= CHUNK {
        host_log_bytes(s.as_bytes());
    } else {
        for chunk in s.as_bytes().chunks(CHUNK) {
            host_log_bytes(chunk);
        }
    }
}

/// `guest_log!` works like `println!`, but routes text to the host logger.
/// Usage: `guest_log!("parsed: a={}, b={}", a, b);`
#[macro_export]
macro_rules! guest_log {
    ($($arg:tt)*) => {{
        let __s = ::std::format!($($arg)*);
        $crate::host_log(&__s);
    }};
}

pub fn host_http_json<T: serde::de::DeserializeOwned>(
    job: &serde_json::Value,
) -> Result<T, String> {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        // 1) serialize job JSON
        let bytes = serde_json::to_vec(job).map_err(|e| e.to_string())?;

        // 2) put request bytes in guest memory
        let in_ptr = alloc(bytes.len());
        if in_ptr.is_null() {
            return Err("alloc failed".into());
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), in_ptr, bytes.len());

        // 3) call host; host awaits HTTP and resumes us; we get packed (ptr,len)
        let packed = http_request(in_ptr as i32, bytes.len() as i32);

        // free input
        dealloc(in_ptr, bytes.len());

        // 4) unpack & read response
        let slice = WasmSlice::unpack(packed);
        let out_slice = std::slice::from_raw_parts(slice.ptr as *const u8, slice.len as usize);
        let result: T =
            serde_json::from_slice(out_slice).map_err(|e| format!("json decode error: {e}"))?;

        // host used our `alloc` to create the buffer — free it
        dealloc(slice.ptr as *mut u8, slice.len as usize);

        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native/testing fallback: stub a response
        eprintln!("[host_http_json stub] job={}", job);
        serde_json::from_value::<T>(serde_json::json!({
            "status": 200,
            "body": "<stub>"
        }))
        .map_err(|e| e.to_string())
    }
}

#[macro_export]
macro_rules! host_http {
    ($($json:tt)*) => {{
        let job = ::serde_json::json!($($json)*);
        $crate::host_http_json(&job)
    }};
}