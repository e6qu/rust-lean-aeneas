pub mod coordinator;
pub mod specialist;
pub mod critic;

use crate::agent_trait::{AgentInstance, AgentKind, AgentState};

/// Dispatch processing to the appropriate agent handler based on AgentKind.
/// Moves envelopes from inbox, processes them, and places results in outbox.
pub fn agent_process(agent: &mut AgentInstance) {
    if agent.state == AgentState::Finished || agent.state == AgentState::Failed {
        return;
    }
    if agent.inbox.is_empty() {
        return;
    }
    agent.state = AgentState::Busy;
    let envelopes = core::mem::take(&mut agent.inbox);
    for env in &envelopes {
        match &mut agent.kind {
            AgentKind::Coordinator(cs) => {
                let out = coordinator::coordinator_process(cs, env, agent.id);
                agent.outbox.extend(out);
            }
            AgentKind::Specialist(ss) => {
                let out = specialist::specialist_process(ss, env, agent.id);
                agent.outbox.extend(out);
            }
            AgentKind::Critic(cs) => {
                let out = critic::critic_process(cs, env, agent.id);
                agent.outbox.extend(out);
            }
        }
    }
    agent.state = AgentState::Ready;
}

/// Check if an agent is active (not Finished or Failed).
pub fn is_agent_active(agent: &AgentInstance) -> bool {
    agent.state != AgentState::Finished && agent.state != AgentState::Failed
}
