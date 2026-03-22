use crate::agent_trait::{AgentId, CriticState, Envelope, Message, MessageKind, Recipient};

/// Process an incoming envelope as a critic.
///
/// A critic reviews content and produces a review message.
pub fn critic_process(
    state: &mut CriticState,
    envelope: &Envelope,
    self_id: AgentId,
) -> Vec<Envelope> {
    let mut outbox = Vec::new();
    match envelope.message.kind {
        MessageKind::Delegation | MessageKind::Task | MessageKind::Response => {
            // Review the content
            state.reviews_given += 1;

            // Send a review back to the sender
            outbox.push(Envelope {
                sender: self_id,
                recipient: Recipient::Direct(envelope.sender),
                message: Message {
                    kind: MessageKind::Review,
                    content_id: envelope.message.content_id,
                },
                sequence_num: 0,
            });
        }
        MessageKind::Vote => {
            // Critics always vote based on their criteria count
            let vote_value = if state.criteria.len() > 0 { 1 } else { 0 };
            outbox.push(Envelope {
                sender: self_id,
                recipient: Recipient::Direct(envelope.sender),
                message: Message {
                    kind: MessageKind::Vote,
                    content_id: vote_value as u32,
                },
                sequence_num: 0,
            });
        }
        _ => {}
    }
    outbox
}
