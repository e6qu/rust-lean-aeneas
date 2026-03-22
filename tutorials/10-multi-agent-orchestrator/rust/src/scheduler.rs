use crate::agent_trait::AgentId;

/// The scheduling policy.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SchedulerKind {
    RoundRobin,
    Priority,
}

/// Scheduler state tracking execution fairness.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scheduler {
    pub kind: SchedulerKind,
    pub agent_ids: Vec<AgentId>,
    pub current_index: u32,
    pub turns_given: Vec<u32>,
}

/// Create a new round-robin scheduler for the given agents.
pub fn scheduler_round_robin(agent_ids: Vec<AgentId>) -> Scheduler {
    let n = agent_ids.len();
    Scheduler {
        kind: SchedulerKind::RoundRobin,
        agent_ids,
        current_index: 0,
        turns_given: vec![0; n],
    }
}

/// Create a new priority scheduler. Agents earlier in the list have higher priority.
pub fn scheduler_priority(agent_ids: Vec<AgentId>) -> Scheduler {
    let n = agent_ids.len();
    Scheduler {
        kind: SchedulerKind::Priority,
        agent_ids,
        current_index: 0,
        turns_given: vec![0; n],
    }
}

/// Return the next agent to run and advance the scheduler.
/// Returns `None` if there are no agents.
pub fn next_agent(sched: &mut Scheduler) -> Option<AgentId> {
    if sched.agent_ids.is_empty() {
        return None;
    }
    match sched.kind {
        SchedulerKind::RoundRobin => {
            let idx = sched.current_index as usize;
            let agent_id = sched.agent_ids[idx];
            sched.turns_given[idx] += 1;
            sched.current_index = ((sched.current_index + 1) as usize % sched.agent_ids.len()) as u32;
            Some(agent_id)
        }
        SchedulerKind::Priority => {
            // Priority: always pick the first agent (highest priority).
            // In a more sophisticated version, we could deprioritize based on turns.
            // Here we pick the agent with the fewest turns to ensure some fairness,
            // breaking ties by index (lower index = higher priority).
            let mut best_idx = 0;
            let mut best_turns = sched.turns_given[0];
            for i in 1..sched.agent_ids.len() {
                if sched.turns_given[i] < best_turns {
                    best_turns = sched.turns_given[i];
                    best_idx = i;
                }
            }
            let agent_id = sched.agent_ids[best_idx];
            sched.turns_given[best_idx] += 1;
            Some(agent_id)
        }
    }
}

/// Returns the number of turns given to a specific agent.
pub fn turns_for_agent(sched: &Scheduler, agent_id: AgentId) -> u32 {
    for i in 0..sched.agent_ids.len() {
        if sched.agent_ids[i] == agent_id {
            return sched.turns_given[i];
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin_cycles() {
        let mut sched = scheduler_round_robin(vec![10, 20, 30]);
        assert_eq!(next_agent(&mut sched), Some(10));
        assert_eq!(next_agent(&mut sched), Some(20));
        assert_eq!(next_agent(&mut sched), Some(30));
        assert_eq!(next_agent(&mut sched), Some(10));
    }

    #[test]
    fn test_round_robin_fairness() {
        let agents = vec![1, 2, 3];
        let k = agents.len() as u32;
        let n = 5u32;
        let mut sched = scheduler_round_robin(agents.clone());
        for _ in 0..(n * k) {
            next_agent(&mut sched);
        }
        for agent_id in &agents {
            assert_eq!(turns_for_agent(&sched, *agent_id), n);
        }
    }

    #[test]
    fn test_priority_fairness() {
        let mut sched = scheduler_priority(vec![1, 2, 3]);
        // With min-turns priority, all agents should get turns
        for _ in 0..9 {
            next_agent(&mut sched);
        }
        assert_eq!(sched.turns_given[0], 3);
        assert_eq!(sched.turns_given[1], 3);
        assert_eq!(sched.turns_given[2], 3);
    }

    #[test]
    fn test_empty_scheduler() {
        let mut sched = scheduler_round_robin(vec![]);
        assert_eq!(next_agent(&mut sched), None);
    }
}
