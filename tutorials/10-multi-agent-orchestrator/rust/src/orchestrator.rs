use crate::agent_trait::{AgentId, AgentInstance, AgentState, Envelope, Recipient};
use crate::agents::{agent_process, is_agent_active};
use crate::message_bus::{bus_is_empty, bus_send, bus_deliver};
use crate::router::{resolve_recipient, Router};
use crate::scheduler::{next_agent, Scheduler, SchedulerKind};

/// Configuration limits for the orchestrator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OrchestratorConfig {
    pub turn_budget: u32,
    pub scheduler_kind: SchedulerKind,
}

/// The full orchestrator state, composing all subsystems.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OrchestratorState {
    pub agents: Vec<AgentInstance>,
    pub bus: crate::message_bus::MessageBus,
    pub router: Router,
    pub scheduler: Scheduler,
    pub turn_count: u32,
    pub config: OrchestratorConfig,
}

/// Find an agent by ID and return its index.
fn find_agent_index(agents: &[AgentInstance], agent_id: AgentId) -> Option<usize> {
    for i in 0..agents.len() {
        if agents[i].id == agent_id {
            return Some(i);
        }
    }
    None
}

/// Collect all agent IDs from the agent list.
fn all_agent_ids(agents: &[AgentInstance]) -> Vec<AgentId> {
    agents.iter().map(|a| a.id).collect()
}

/// Perform one orchestrator step:
/// 1. Pick the next agent via the scheduler.
/// 2. Deliver messages from the bus to that agent's inbox.
/// 3. Run the agent's process function.
/// 4. Collect the agent's outbox and route messages back onto the bus.
/// 5. Increment the turn counter.
pub fn orchestrator_step(state: &mut OrchestratorState) {
    // Pick next agent
    let agent_id = match next_agent(&mut state.scheduler) {
        Some(id) => id,
        None => return,
    };

    let agent_idx = match find_agent_index(&state.agents, agent_id) {
        Some(idx) => idx,
        None => return,
    };

    // Skip inactive agents
    if !is_agent_active(&state.agents[agent_idx]) {
        state.turn_count += 1;
        return;
    }

    // Deliver messages from bus to agent inbox
    let delivered = bus_deliver(&mut state.bus, agent_id);
    state.agents[agent_idx].inbox.extend(delivered);

    // Process the agent
    agent_process(&mut state.agents[agent_idx]);

    // Collect outbox and route messages onto the bus
    let outbox = core::mem::take(&mut state.agents[agent_idx].outbox);
    let ids = all_agent_ids(&state.agents);
    for env in outbox {
        // Resolve the recipient to expand Topic/Broadcast into Direct messages
        let targets = resolve_recipient(&state.router, &env.recipient, &ids);
        for target_id in targets {
            let routed_env = Envelope {
                sender: env.sender,
                recipient: Recipient::Direct(target_id),
                message: env.message.clone(),
                sequence_num: 0,
            };
            bus_send(&mut state.bus, routed_env);
        }
    }

    state.turn_count += 1;
}

/// Run the orchestrator until the turn budget is exhausted or no agents are active.
pub fn orchestrator_run(state: &mut OrchestratorState) {
    while state.turn_count < state.config.turn_budget {
        if !has_active_agents(state) && bus_is_empty(&state.bus) {
            break;
        }
        orchestrator_step(state);
    }
}

/// Returns true if any agent is still active (Ready or Busy).
pub fn has_active_agents(state: &OrchestratorState) -> bool {
    for agent in &state.agents {
        if is_agent_active(agent) {
            return true;
        }
    }
    false
}

/// Returns the number of agents in Finished state.
pub fn finished_agent_count(state: &OrchestratorState) -> u32 {
    state
        .agents
        .iter()
        .filter(|a| a.state == AgentState::Finished)
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_trait::*;
    use crate::message_bus::bus_new;
    use crate::router::router_new;
    use crate::scheduler::scheduler_round_robin;

    fn make_specialist(id: AgentId, specialty: u32) -> AgentInstance {
        AgentInstance {
            id,
            kind: AgentKind::Specialist(SpecialistState {
                specialty,
                context: Vec::new(),
                steps_taken: 0,
            }),
            state: AgentState::Ready,
            inbox: Vec::new(),
            outbox: Vec::new(),
        }
    }

    fn make_coordinator(id: AgentId, managed: Vec<AgentId>) -> AgentInstance {
        AgentInstance {
            id,
            kind: AgentKind::Coordinator(CoordinatorState {
                protocol: ProtocolKind::RequestResponse,
                managed_agents: managed,
                pending_responses: 0,
                round: 0,
            }),
            state: AgentState::Ready,
            inbox: Vec::new(),
            outbox: Vec::new(),
        }
    }

    fn make_critic(id: AgentId, criteria: Vec<u32>) -> AgentInstance {
        AgentInstance {
            id,
            kind: AgentKind::Critic(CriticState {
                criteria,
                reviews_given: 0,
            }),
            state: AgentState::Ready,
            inbox: Vec::new(),
            outbox: Vec::new(),
        }
    }

    #[test]
    fn test_orchestrator_budget() {
        let agents = vec![make_specialist(1, 0), make_specialist(2, 1)];
        let agent_ids = vec![1, 2];
        let mut state = OrchestratorState {
            agents,
            bus: bus_new(),
            router: router_new(),
            scheduler: scheduler_round_robin(agent_ids),
            turn_count: 0,
            config: OrchestratorConfig {
                turn_budget: 10,
                scheduler_kind: SchedulerKind::RoundRobin,
            },
        };

        // Seed one message
        let env = Envelope {
            sender: 0,
            recipient: Recipient::Direct(1),
            message: Message {
                kind: MessageKind::Task,
                content_id: 42,
            },
            sequence_num: 0,
        };
        bus_send(&mut state.bus, env);

        orchestrator_run(&mut state);
        assert!(state.turn_count <= state.config.turn_budget);
    }

    #[test]
    fn test_orchestrator_delegation() {
        // Coordinator delegates to two specialists
        let agents = vec![
            make_coordinator(1, vec![2, 3]),
            make_specialist(2, 10),
            make_specialist(3, 20),
        ];
        let agent_ids = vec![1, 2, 3];
        let mut state = OrchestratorState {
            agents,
            bus: bus_new(),
            router: router_new(),
            scheduler: scheduler_round_robin(agent_ids),
            turn_count: 0,
            config: OrchestratorConfig {
                turn_budget: 20,
                scheduler_kind: SchedulerKind::RoundRobin,
            },
        };

        // Send a task to the coordinator
        let env = Envelope {
            sender: 0,
            recipient: Recipient::Direct(1),
            message: Message {
                kind: MessageKind::Task,
                content_id: 100,
            },
            sequence_num: 0,
        };
        bus_send(&mut state.bus, env);

        orchestrator_run(&mut state);
        assert!(state.turn_count <= 20);
        // Check that the bus processed messages
        assert!(state.bus.delivered.len() > 0);
    }

    #[test]
    fn test_orchestrator_with_critic() {
        let agents = vec![
            make_coordinator(1, vec![2, 3]),
            make_specialist(2, 10),
            make_critic(3, vec![1, 2, 3]),
        ];
        let agent_ids = vec![1, 2, 3];
        let mut state = OrchestratorState {
            agents,
            bus: bus_new(),
            router: router_new(),
            scheduler: scheduler_round_robin(agent_ids),
            turn_count: 0,
            config: OrchestratorConfig {
                turn_budget: 30,
                scheduler_kind: SchedulerKind::RoundRobin,
            },
        };

        let env = Envelope {
            sender: 0,
            recipient: Recipient::Direct(1),
            message: Message {
                kind: MessageKind::Task,
                content_id: 200,
            },
            sequence_num: 0,
        };
        bus_send(&mut state.bus, env);

        orchestrator_run(&mut state);
        assert!(state.turn_count <= 30);
    }

    #[test]
    fn test_no_active_agents_stops() {
        let mut agents = vec![make_specialist(1, 0)];
        agents[0].state = AgentState::Finished;
        let mut state = OrchestratorState {
            agents,
            bus: bus_new(),
            router: router_new(),
            scheduler: scheduler_round_robin(vec![1]),
            turn_count: 0,
            config: OrchestratorConfig {
                turn_budget: 100,
                scheduler_kind: SchedulerKind::RoundRobin,
            },
        };
        orchestrator_run(&mut state);
        // Should stop immediately since no active agents and bus is empty
        assert_eq!(state.turn_count, 0);
    }

    #[test]
    fn test_has_active_agents() {
        let agents = vec![make_specialist(1, 0), make_specialist(2, 1)];
        let state = OrchestratorState {
            agents,
            bus: bus_new(),
            router: router_new(),
            scheduler: scheduler_round_robin(vec![1, 2]),
            turn_count: 0,
            config: OrchestratorConfig {
                turn_budget: 10,
                scheduler_kind: SchedulerKind::RoundRobin,
            },
        };
        assert!(has_active_agents(&state));
    }
}
