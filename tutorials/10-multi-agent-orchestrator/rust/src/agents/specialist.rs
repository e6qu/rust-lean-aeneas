use crate::agent_trait::{AgentId, Envelope, Message, MessageKind, Recipient, SpecialistState};

/// Process an incoming envelope as a specialist.
///
/// A specialist handles delegated tasks by producing a response.
/// It tracks how many steps it has taken in its context.
pub fn specialist_process(
    state: &mut SpecialistState,
    envelope: &Envelope,
    self_id: AgentId,
) -> Vec<Envelope> {
    let mut outbox = Vec::new();
    match envelope.message.kind {
        MessageKind::Delegation | MessageKind::Task => {
            // Process the task: increment steps, add to context
            state.steps_taken += 1;
            state.context.push(envelope.message.content_id);

            // Send a response back to the sender
            outbox.push(Envelope {
                sender: self_id,
                recipient: Recipient::Direct(envelope.sender),
                message: Message {
                    kind: MessageKind::Response,
                    content_id: envelope.message.content_id + state.specialty,
                },
                sequence_num: 0,
            });
        }
        MessageKind::Vote => {
            // Cast a vote (specialist votes based on specialty parity)
            outbox.push(Envelope {
                sender: self_id,
                recipient: Recipient::Direct(envelope.sender),
                message: Message {
                    kind: MessageKind::Vote,
                    content_id: if state.specialty.is_multiple_of(2) {
                        1
                    } else {
                        0
                    },
                },
                sequence_num: 0,
            });
        }
        _ => {}
    }
    outbox
}
