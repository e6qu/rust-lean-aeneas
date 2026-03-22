use crate::agent_trait::{
    AgentId, CoordinatorState, Envelope, Message, MessageKind, ProtocolKind, Recipient,
};

/// Process an incoming envelope as a coordinator.
///
/// The coordinator delegates tasks to managed agents and tracks responses.
/// When all responses are collected, it sends a Completion message.
pub fn coordinator_process(
    state: &mut CoordinatorState,
    envelope: &Envelope,
    self_id: AgentId,
) -> Vec<Envelope> {
    let mut outbox = Vec::new();
    match envelope.message.kind {
        MessageKind::Task => {
            // Delegate the task to all managed agents
            state.pending_responses = state.managed_agents.len() as u32;
            state.round += 1;
            for &agent_id in &state.managed_agents {
                outbox.push(Envelope {
                    sender: self_id,
                    recipient: Recipient::Direct(agent_id),
                    message: Message {
                        kind: MessageKind::Delegation,
                        content_id: envelope.message.content_id,
                    },
                    sequence_num: 0,
                });
            }
        }
        MessageKind::Response => {
            // Collect a response from a managed agent
            if state.pending_responses > 0 {
                state.pending_responses -= 1;
            }
            if state.pending_responses == 0 {
                // All responses collected; send completion back to the original sender
                outbox.push(Envelope {
                    sender: self_id,
                    recipient: Recipient::Direct(envelope.sender),
                    message: Message {
                        kind: MessageKind::Completion,
                        content_id: envelope.message.content_id,
                    },
                    sequence_num: 0,
                });
            }
        }
        MessageKind::Review => {
            // Forward reviews based on protocol
            match state.protocol {
                ProtocolKind::Debate => {
                    // In debate, broadcast the review to all managed agents
                    outbox.push(Envelope {
                        sender: self_id,
                        recipient: Recipient::Broadcast,
                        message: Message {
                            kind: MessageKind::Review,
                            content_id: envelope.message.content_id,
                        },
                        sequence_num: 0,
                    });
                }
                ProtocolKind::Voting => {
                    // In voting, forward vote request to all managed agents
                    for &agent_id in &state.managed_agents {
                        outbox.push(Envelope {
                            sender: self_id,
                            recipient: Recipient::Direct(agent_id),
                            message: Message {
                                kind: MessageKind::Vote,
                                content_id: envelope.message.content_id,
                            },
                            sequence_num: 0,
                        });
                    }
                    state.pending_responses = state.managed_agents.len() as u32;
                }
                _ => {}
            }
        }
        _ => {}
    }
    outbox
}
