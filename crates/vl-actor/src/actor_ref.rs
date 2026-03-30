use crate::{ActorMsg, TbActorId};
use tokio::sync::mpsc;

/// Handle to send messages to a running actor.
///
/// Mirrors ThingsBoard's `TbActorRef` — provides `tell` (normal priority)
/// and `tell_high_priority` (high priority) methods.
#[derive(Clone)]
pub struct TbActorRef {
    id: TbActorId,
    high_tx: mpsc::UnboundedSender<ActorMsg>,
    normal_tx: mpsc::UnboundedSender<ActorMsg>,
}

impl TbActorRef {
    pub(crate) fn new(
        id: TbActorId,
        high_tx: mpsc::UnboundedSender<ActorMsg>,
        normal_tx: mpsc::UnboundedSender<ActorMsg>,
    ) -> Self {
        Self {
            id,
            high_tx,
            normal_tx,
        }
    }

    /// Actor ID this ref points to.
    pub fn actor_id(&self) -> &TbActorId {
        &self.id
    }

    /// Send a message with normal priority.
    pub fn tell(&self, msg: ActorMsg) {
        let _ = self.normal_tx.send(msg);
    }

    /// Send a message with high priority (processed before normal).
    pub fn tell_high_priority(&self, msg: ActorMsg) {
        let _ = self.high_tx.send(msg);
    }

    /// Send with auto-detected priority based on `msg.is_high_priority()`.
    pub fn tell_auto(&self, msg: ActorMsg) {
        if msg.is_high_priority() {
            self.tell_high_priority(msg);
        } else {
            self.tell(msg);
        }
    }
}

impl std::fmt::Debug for TbActorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TbActorRef")
            .field("id", &self.id)
            .finish()
    }
}
