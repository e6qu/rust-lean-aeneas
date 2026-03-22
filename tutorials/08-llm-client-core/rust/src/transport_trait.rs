use crate::request::Request;
use crate::response::Response;

/// Errors that can occur during transport.
#[derive(Clone, PartialEq, Debug)]
pub enum TransportError {
    ConnectionFailed,
    Timeout,
    InvalidResponse,
}

/// Trait for sending LLM requests and receiving responses.
///
/// The pure core never provides an implementation of this trait.
/// In Lean, this becomes a structure with function fields, allowing
/// specifications to describe what a conforming transport must satisfy
/// without providing a concrete implementation.
pub trait LlmTransport {
    fn send_request(&mut self, req: &Request) -> Result<Response, TransportError>;
}
