// deps/mod.rs — Stub types representing components from Tutorials 05-10.
//
// In a real workspace these would be imported from sibling crates.  Here we
// provide minimal stand-ins so the verified_core compiles independently.

// ── Tutorial 05: Message Protocol ──────────────────────────────────────────

/// Identifies an agent within the orchestrator.
pub type AgentId = u32;

/// A serialised message payload (simplified from the full protocol).
pub type Payload = Vec<u8>;

/// An envelope carrying a message between agents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub sender: AgentId,
    pub recipient: AgentId,
    pub content_id: u32,
}

// ── Tutorial 06: Buffer Management ─────────────────────────────────────────

/// A simplified input buffer (the real version is a gap buffer).
pub type InputBuffer = Vec<u8>;

// ── Tutorial 07: TUI Core ──────────────────────────────────────────────────

/// Rectangle describing a pane's screen area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

// ── Tutorial 08: LLM Client Core ──────────────────────────────────────────

/// A single chat message (role + content).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: u32,     // 0 = user, 1 = assistant, 2 = system
    pub content_id: u32,
}

/// Conversation context for one agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
}

// ── Tutorial 09: Agent Reasoning ──────────────────────────────────────────

/// Agent phase within the reasoning state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentPhase {
    Idle,
    Thinking,
    Acting,
    Done,
}

// ── Tutorial 10: Multi-Agent Orchestrator ──────────────────────────────────

/// Simplified message bus: a FIFO queue of envelopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageBus {
    pub queue: Vec<Envelope>,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus { queue: Vec::new() }
    }

    pub fn send(&mut self, envelope: Envelope) {
        self.queue.push(envelope);
    }

    pub fn deliver(&mut self) -> Option<Envelope> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
