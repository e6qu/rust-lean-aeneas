// shell/http_client.rs — HTTP client via ureq (stub).
//
// In a real build this module sends HTTP requests to an LLM API endpoint
// and returns the response body.  It is part of the UNVERIFIED shell.
//
// The verified core never calls this directly.  Instead, `app_update`
// produces `SideEffect::SendHttpRequest` descriptors, and the event loop
// calls this module to execute them.

/// Send an HTTP POST request to the configured LLM endpoint.
///
/// `request_body` is a serialised JSON payload (produced by the verified
/// core's message bridge).
///
/// Returns `Ok(response_bytes)` on success or `Err(error_code)` on failure.
///
/// Stub: always returns an empty success response.
pub fn send_llm_request(_endpoint: &str, _request_body: &[u8]) -> Result<Vec<u8>, u32> {
    // Real implementation:
    //   let resp = ureq::post(endpoint)
    //       .set("Content-Type", "application/json")
    //       .send_bytes(request_body)?;
    //   let mut body = Vec::new();
    //   resp.into_reader().read_to_end(&mut body)?;
    //   Ok(body)
    Ok(Vec::new())
}

/// Send a streaming HTTP request (for Server-Sent Events / chunked responses).
///
/// In a real implementation this would return an iterator of chunks.
///
/// Stub: returns an empty iterator.
pub fn send_streaming_request(_endpoint: &str, _request_body: &[u8]) -> Result<Vec<Vec<u8>>, u32> {
    // Real implementation would yield chunks as they arrive.
    Ok(Vec::new())
}
