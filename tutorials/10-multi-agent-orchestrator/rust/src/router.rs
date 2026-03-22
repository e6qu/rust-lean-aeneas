use crate::agent_trait::{AgentId, Recipient};

/// A subscription linking an agent to a topic.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TopicSubscription {
    pub topic_id: u32,
    pub agent_id: AgentId,
}

/// Router that maps topics to subscribing agents.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Router {
    pub subscriptions: Vec<TopicSubscription>,
}

/// Create an empty router.
pub fn router_new() -> Router {
    Router {
        subscriptions: Vec::new(),
    }
}

/// Subscribe an agent to a topic.
pub fn router_subscribe(router: &mut Router, topic_id: u32, agent_id: AgentId) {
    router
        .subscriptions
        .push(TopicSubscription { topic_id, agent_id });
}

/// Resolve a recipient to a list of concrete agent IDs.
///
/// - `Direct(id)` resolves to `[id]` (singleton).
/// - `Broadcast` resolves to all known agent IDs.
/// - `Topic(t)` resolves to the agents subscribed to topic `t`.
pub fn resolve_recipient(
    router: &Router,
    recipient: &Recipient,
    all_agent_ids: &[AgentId],
) -> Vec<AgentId> {
    match recipient {
        Recipient::Direct(id) => vec![*id],
        Recipient::Broadcast => all_agent_ids.to_vec(),
        Recipient::Topic(topic_id) => {
            let mut result = Vec::new();
            for sub in &router.subscriptions {
                if sub.topic_id == *topic_id {
                    result.push(sub.agent_id);
                }
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_direct() {
        let router = router_new();
        let agents = vec![1, 2, 3];
        let result = resolve_recipient(&router, &Recipient::Direct(2), &agents);
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_resolve_broadcast() {
        let router = router_new();
        let agents = vec![1, 2, 3];
        let result = resolve_recipient(&router, &Recipient::Broadcast, &agents);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_resolve_topic() {
        let mut router = router_new();
        router_subscribe(&mut router, 42, 1);
        router_subscribe(&mut router, 42, 3);
        router_subscribe(&mut router, 99, 2);
        let agents = vec![1, 2, 3];
        let result = resolve_recipient(&router, &Recipient::Topic(42), &agents);
        assert_eq!(result, vec![1, 3]);
    }

    #[test]
    fn test_resolve_topic_empty() {
        let router = router_new();
        let agents = vec![1, 2, 3];
        let result = resolve_recipient(&router, &Recipient::Topic(42), &agents);
        assert!(result.is_empty());
    }
}
