/// Unique identifier for each agent instance.
pub type AgentId = u32;

/// Lifecycle state of an agent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentState {
    Ready,
    Busy,
    Finished,
    Failed,
}

/// The kind (type tag) of a message payload.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MessageKind {
    Task,
    Response,
    Review,
    Vote,
    Delegation,
    Completion,
    Error,
}

/// A message payload with a kind tag and a content identifier.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub content_id: u32,
}

/// Addressing modes for message delivery.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Recipient {
    Direct(AgentId),
    Broadcast,
    Topic(u32),
}

/// A sequenced envelope wrapping a message for routing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Envelope {
    pub sender: AgentId,
    pub recipient: Recipient,
    pub message: Message,
    pub sequence_num: u32,
}

/// Protocol kinds supported by the coordinator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProtocolKind {
    RequestResponse,
    Pipeline,
    Debate,
    Voting,
}

/// State held by a coordinator agent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CoordinatorState {
    pub protocol: ProtocolKind,
    pub managed_agents: Vec<AgentId>,
    pub pending_responses: u32,
    pub round: u32,
}

/// State held by a specialist agent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SpecialistState {
    pub specialty: u32,
    pub context: Vec<u32>,
    pub steps_taken: u32,
}

/// State held by a critic agent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CriticState {
    pub criteria: Vec<u32>,
    pub reviews_given: u32,
}

/// Enum-dispatch agent kind. Each variant holds agent-specific state.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AgentKind {
    Coordinator(CoordinatorState),
    Specialist(SpecialistState),
    Critic(CriticState),
}

/// A running agent instance.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AgentInstance {
    pub id: AgentId,
    pub kind: AgentKind,
    pub state: AgentState,
    pub inbox: Vec<Envelope>,
    pub outbox: Vec<Envelope>,
}
