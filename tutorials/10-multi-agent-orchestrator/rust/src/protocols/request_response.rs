use crate::agent_trait::AgentId;

/// Tracks the state of a simple request-response exchange.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RRState {
    Idle,
    AwaitingResponse,
    Completed,
    TimedOut,
}

/// A request-response protocol instance.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RequestResponse {
    pub requester: AgentId,
    pub responder: AgentId,
    pub request_id: u32,
    pub state: RRState,
    pub ticks_waiting: u32,
    pub timeout: u32,
}

/// Create a new request-response protocol instance.
pub fn rr_new(requester: AgentId, responder: AgentId, request_id: u32, timeout: u32) -> RequestResponse {
    RequestResponse {
        requester,
        responder,
        request_id,
        state: RRState::Idle,
        ticks_waiting: 0,
        timeout,
    }
}

/// Mark the request as sent; transition to AwaitingResponse.
pub fn rr_send(rr: &mut RequestResponse) {
    if rr.state == RRState::Idle {
        rr.state = RRState::AwaitingResponse;
    }
}

/// Mark the response as received; transition to Completed.
pub fn rr_receive(rr: &mut RequestResponse) {
    if rr.state == RRState::AwaitingResponse {
        rr.state = RRState::Completed;
    }
}

/// Advance one tick while awaiting response. Times out if budget exceeded.
pub fn rr_tick(rr: &mut RequestResponse) {
    if rr.state == RRState::AwaitingResponse {
        rr.ticks_waiting += 1;
        if rr.ticks_waiting >= rr.timeout {
            rr.state = RRState::TimedOut;
        }
    }
}

/// Returns true if the protocol has completed (either success or timeout).
pub fn rr_is_done(rr: &RequestResponse) -> bool {
    rr.state == RRState::Completed || rr.state == RRState::TimedOut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rr_lifecycle() {
        let mut rr = rr_new(1, 2, 100, 5);
        assert_eq!(rr.state, RRState::Idle);
        rr_send(&mut rr);
        assert_eq!(rr.state, RRState::AwaitingResponse);
        rr_receive(&mut rr);
        assert_eq!(rr.state, RRState::Completed);
        assert!(rr_is_done(&rr));
    }

    #[test]
    fn test_rr_timeout() {
        let mut rr = rr_new(1, 2, 100, 3);
        rr_send(&mut rr);
        for _ in 0..3 {
            rr_tick(&mut rr);
        }
        assert_eq!(rr.state, RRState::TimedOut);
        assert!(rr_is_done(&rr));
    }
}
