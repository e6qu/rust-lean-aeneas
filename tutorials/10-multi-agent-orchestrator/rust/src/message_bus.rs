use crate::agent_trait::{AgentId, Envelope, Recipient};

/// Central message transport with an audit trail.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MessageBus {
    pub queue: Vec<Envelope>,
    pub delivered: Vec<Envelope>,
    pub next_seq: u32,
}

/// Create an empty message bus.
pub fn bus_new() -> MessageBus {
    MessageBus {
        queue: Vec::new(),
        delivered: Vec::new(),
        next_seq: 0,
    }
}

/// Enqueue an envelope, assigning it the next sequence number.
pub fn bus_send(bus: &mut MessageBus, mut env: Envelope) {
    env.sequence_num = bus.next_seq;
    bus.next_seq += 1;
    bus.queue.push(env);
}

/// Check whether an envelope targets a given agent.
fn envelope_targets(env: &Envelope, agent_id: AgentId) -> bool {
    match &env.recipient {
        Recipient::Direct(id) => *id == agent_id,
        Recipient::Broadcast => true,
        Recipient::Topic(_) => {
            // Topic resolution is handled by the router before delivery;
            // by the time envelopes reach the bus with Topic addressing,
            // they should have been expanded. We return false here.
            false
        }
    }
}

/// Deliver all envelopes targeting `agent_id`: move them from queue to delivered.
/// Returns the delivered envelopes.
pub fn bus_deliver(bus: &mut MessageBus, agent_id: AgentId) -> Vec<Envelope> {
    let mut result = Vec::new();
    let mut remaining = Vec::new();
    // We iterate through the queue, separating matching from non-matching.
    let queue = core::mem::take(&mut bus.queue);
    for env in queue {
        if envelope_targets(&env, agent_id) {
            result.push(env.clone());
            bus.delivered.push(env);
        } else {
            remaining.push(env);
        }
    }
    bus.queue = remaining;
    result
}

/// Dequeue the next envelope from the bus (FIFO), moving it to delivered.
/// Returns None if the queue is empty.
pub fn bus_deliver_next(bus: &mut MessageBus) -> Option<Envelope> {
    if bus.queue.is_empty() {
        return None;
    }
    let env = bus.queue.remove(0);
    bus.delivered.push(env.clone());
    Some(env)
}

/// Returns true if the bus queue is empty.
pub fn bus_is_empty(bus: &MessageBus) -> bool {
    bus.queue.is_empty()
}

/// Returns the total number of envelopes that have been delivered.
pub fn bus_delivered_count(bus: &MessageBus) -> u32 {
    bus.delivered.len() as u32
}

/// Returns the total number of envelopes still in the queue.
pub fn bus_pending_count(bus: &MessageBus) -> u32 {
    bus.queue.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_trait::{Message, MessageKind};

    fn make_envelope(sender: AgentId, recipient: Recipient, content_id: u32) -> Envelope {
        Envelope {
            sender,
            recipient,
            message: Message {
                kind: MessageKind::Task,
                content_id,
            },
            sequence_num: 0,
        }
    }

    #[test]
    fn test_bus_send_assigns_sequence_numbers() {
        let mut bus = bus_new();
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(2), 100));
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(3), 101));
        assert_eq!(bus.queue.len(), 2);
        assert_eq!(bus.queue[0].sequence_num, 0);
        assert_eq!(bus.queue[1].sequence_num, 1);
        assert_eq!(bus.next_seq, 2);
    }

    #[test]
    fn test_bus_deliver_moves_to_delivered() {
        let mut bus = bus_new();
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(2), 100));
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(3), 101));
        bus_send(&mut bus, make_envelope(1, Recipient::Broadcast, 102));

        let delivered = bus_deliver(&mut bus, 2);
        // Agent 2 should receive Direct(2) and Broadcast
        assert_eq!(delivered.len(), 2);
        assert_eq!(delivered[0].message.content_id, 100);
        assert_eq!(delivered[1].message.content_id, 102);
        // Queue should have only the Direct(3) envelope left
        assert_eq!(bus.queue.len(), 1);
        assert_eq!(bus.delivered.len(), 2);
    }

    #[test]
    fn test_bus_deliver_next_fifo() {
        let mut bus = bus_new();
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(2), 100));
        bus_send(&mut bus, make_envelope(1, Recipient::Direct(3), 101));

        let first = bus_deliver_next(&mut bus).unwrap();
        assert_eq!(first.message.content_id, 100);
        assert_eq!(bus.queue.len(), 1);
        assert_eq!(bus.delivered.len(), 1);

        let second = bus_deliver_next(&mut bus).unwrap();
        assert_eq!(second.message.content_id, 101);
        assert!(bus_deliver_next(&mut bus).is_none());
    }

    #[test]
    fn test_bus_empty() {
        let bus = bus_new();
        assert!(bus_is_empty(&bus));
        assert_eq!(bus_pending_count(&bus), 0);
        assert_eq!(bus_delivered_count(&bus), 0);
    }
}
