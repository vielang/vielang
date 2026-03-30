use crate::{
    ActorError, ActorMsg, EntityType, TbActor, TbActorId, TbActorRef,
    mailbox::{MailboxSettings, TbActorMailbox},
};
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Configuration for the actor system.
#[derive(Debug, Clone)]
pub struct TbActorSystemSettings {
    /// Max messages per batch before yielding.
    pub actor_throughput: usize,
    /// Max init retry attempts (0 = unlimited).
    pub max_init_attempts: u32,
}

impl Default for TbActorSystemSettings {
    fn default() -> Self {
        Self {
            actor_throughput: 30,
            max_init_attempts: 10,
        }
    }
}

/// Context passed to actors for child management and message routing.
///
/// Mirrors ThingsBoard's `TbActorCtx`.
#[derive(Clone)]
pub struct TbActorCtx {
    self_id: TbActorId,
    system: Arc<ActorSystemInner>,
}

impl TbActorCtx {
    /// This actor's ID.
    pub fn self_id(&self) -> &TbActorId {
        &self.self_id
    }

    /// Reference to self for sending self-messages.
    pub fn self_ref(&self) -> Option<TbActorRef> {
        self.system.actors.get(&self.self_id).map(|r| r.clone())
    }

    /// Get reference to parent actor.
    pub fn parent_ref(&self) -> Option<TbActorRef> {
        let parents = self.system.child_to_parent.read().ok()?;
        let parent_id = parents.get(&self.self_id)?;
        self.system.actors.get(parent_id).map(|r| r.clone())
    }

    /// Send a message to a target actor by ID.
    pub fn tell(&self, target: &TbActorId, msg: ActorMsg) {
        if let Some(actor_ref) = self.system.actors.get(target) {
            actor_ref.tell(msg);
        } else {
            warn!("tell: actor {} not found", target);
        }
    }

    /// Send a high-priority message to a target actor.
    pub fn tell_high_priority(&self, target: &TbActorId, msg: ActorMsg) {
        if let Some(actor_ref) = self.system.actors.get(target) {
            actor_ref.tell_high_priority(msg);
        } else {
            warn!("tell_high_priority: actor {} not found", target);
        }
    }

    /// Get or create a child actor. If it already exists, return existing ref.
    pub fn get_or_create_child<F>(
        &self,
        child_id: TbActorId,
        creator: F,
    ) -> Result<TbActorRef, ActorError>
    where
        F: FnOnce() -> Box<dyn TbActor>,
    {
        // Check existing first.
        if let Some(r) = self.system.actors.get(&child_id) {
            return Ok(r.clone());
        }

        let actor = creator();
        self.system
            .create_child(child_id, &self.self_id, actor)
    }

    /// Stop a child actor and all its descendants.
    pub fn stop(&self, target: &TbActorId) {
        self.system.stop_actor(target);
    }

    /// Broadcast a message to all children of this actor.
    pub fn broadcast_to_children(&self, msg_factory: &dyn Fn() -> ActorMsg, high_priority: bool) {
        let children = self.system.get_children(&self.self_id);
        for child_id in children {
            if let Some(child_ref) = self.system.actors.get(&child_id) {
                let msg = msg_factory();
                if high_priority {
                    child_ref.tell_high_priority(msg);
                } else {
                    child_ref.tell(msg);
                }
            }
        }
    }

    /// Broadcast to children filtered by entity type.
    pub fn broadcast_to_children_by_type(
        &self,
        entity_type: EntityType,
        msg_factory: &dyn Fn() -> ActorMsg,
        high_priority: bool,
    ) {
        let children = self.system.get_children(&self.self_id);
        for child_id in children {
            if child_id.entity_type() == Some(entity_type) {
                if let Some(child_ref) = self.system.actors.get(&child_id) {
                    let msg = msg_factory();
                    if high_priority {
                        child_ref.tell_high_priority(msg);
                    } else {
                        child_ref.tell(msg);
                    }
                }
            }
        }
    }

    /// Get IDs of children matching a predicate.
    pub fn filter_children(&self, predicate: impl Fn(&TbActorId) -> bool) -> Vec<TbActorId> {
        self.system
            .get_children(&self.self_id)
            .into_iter()
            .filter(predicate)
            .collect()
    }

    /// Look up an actor ref by ID (any actor in the system).
    pub fn get_actor_ref(&self, id: &TbActorId) -> Option<TbActorRef> {
        self.system.actors.get(id).map(|r| r.clone())
    }
}

// ─── Actor System (public API) ──────────────────────────────────

/// The actor system — manages all actors, their hierarchy, and message routing.
///
/// Mirrors ThingsBoard's `DefaultTbActorSystem`.
#[derive(Clone)]
pub struct TbActorSystem {
    inner: Arc<ActorSystemInner>,
}

struct ActorSystemInner {
    settings: TbActorSystemSettings,

    /// All registered actors: actor_id → actor_ref.
    actors: DashMap<TbActorId, TbActorRef>,

    /// Parent → children mapping.
    parent_to_children: DashMap<TbActorId, HashSet<TbActorId>>,

    /// Child → parent mapping.
    child_to_parent: std::sync::RwLock<std::collections::HashMap<TbActorId, TbActorId>>,

    /// Join handles for spawned actor tasks.
    handles: DashMap<TbActorId, tokio::task::JoinHandle<()>>,
}

impl TbActorSystem {
    /// Create a new actor system.
    pub fn new(settings: TbActorSystemSettings) -> Self {
        info!(
            "actor system starting (throughput={}, max_init_attempts={})",
            settings.actor_throughput, settings.max_init_attempts
        );
        Self {
            inner: Arc::new(ActorSystemInner {
                settings,
                actors: DashMap::new(),
                parent_to_children: DashMap::new(),
                child_to_parent: std::sync::RwLock::new(std::collections::HashMap::new()),
                handles: DashMap::new(),
            }),
        }
    }

    /// Create a root actor (no parent).
    pub fn create_root_actor<F>(
        &self,
        id: TbActorId,
        creator: F,
    ) -> Result<TbActorRef, ActorError>
    where
        F: FnOnce() -> Box<dyn TbActor>,
    {
        if let Some(r) = self.inner.actors.get(&id) {
            return Ok(r.clone());
        }

        let actor = creator();
        let ctx = TbActorCtx {
            self_id: id.clone(),
            system: self.inner.clone(),
        };

        let ms = self.inner.mailbox_settings();
        let (actor_ref, handle) = TbActorMailbox::spawn(id.clone(), actor, ctx, ms);

        self.inner.actors.insert(id.clone(), actor_ref.clone());
        self.inner.handles.insert(id.clone(), handle);
        self.inner.parent_to_children.entry(id).or_default();

        debug!("root actor created: {}", actor_ref.actor_id());
        Ok(actor_ref)
    }

    /// Send a message to an actor by ID (auto-detect priority).
    pub fn tell(&self, target: &TbActorId, msg: ActorMsg) {
        if let Some(actor_ref) = self.inner.actors.get(target) {
            actor_ref.tell_auto(msg);
        } else {
            warn!("tell: actor {target} not found");
        }
    }

    /// Send a high-priority message to an actor by ID.
    pub fn tell_high_priority(&self, target: &TbActorId, msg: ActorMsg) {
        if let Some(actor_ref) = self.inner.actors.get(target) {
            actor_ref.tell_high_priority(msg);
        } else {
            warn!("tell_high_priority: actor {target} not found");
        }
    }

    /// Stop an actor and all its descendants.
    pub fn stop(&self, target: &TbActorId) {
        self.inner.stop_actor(target);
    }

    /// Broadcast a message to all children of a given actor.
    pub fn broadcast_to_children(
        &self,
        parent: &TbActorId,
        msg_factory: &dyn Fn() -> ActorMsg,
        high_priority: bool,
    ) {
        let children = self.inner.get_children(parent);
        for child_id in children {
            if let Some(child_ref) = self.inner.actors.get(&child_id) {
                let msg = msg_factory();
                if high_priority {
                    child_ref.tell_high_priority(msg);
                } else {
                    child_ref.tell(msg);
                }
            }
        }
    }

    /// Get an actor ref by ID.
    pub fn get_actor_ref(&self, id: &TbActorId) -> Option<TbActorRef> {
        self.inner.actors.get(id).map(|r| r.clone())
    }

    /// Number of registered actors.
    pub fn actor_count(&self) -> usize {
        self.inner.actors.len()
    }

    /// Shut down all actors gracefully.
    pub async fn shutdown(&self) {
        info!("actor system shutting down ({} actors)", self.actor_count());
        let ids: Vec<TbActorId> = self.inner.actors.iter().map(|r| r.key().clone()).collect();
        for id in &ids {
            self.inner.stop_actor(id);
        }
        // Await remaining handles.
        let handles: Vec<(TbActorId, tokio::task::JoinHandle<()>)> = self
            .inner
            .handles
            .iter()
            .map(|r| (r.key().clone(), ()))
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|(id, _)| self.inner.handles.remove(&id))
            .collect();
        for (id, handle) in handles {
            if let Err(e) = handle.await {
                if !e.is_cancelled() {
                    warn!("actor {id} task panicked: {e}");
                }
            }
        }
        info!("actor system shut down");
    }
}

// ─── Internal methods ───────────────────────────────────────────

impl ActorSystemInner {
    fn mailbox_settings(&self) -> MailboxSettings {
        MailboxSettings {
            actor_throughput: self.settings.actor_throughput,
            max_init_attempts: self.settings.max_init_attempts,
        }
    }

    fn create_child(
        self: &Arc<Self>,
        child_id: TbActorId,
        parent_id: &TbActorId,
        actor: Box<dyn TbActor>,
    ) -> Result<TbActorRef, ActorError> {
        // Double-check if already exists.
        if let Some(r) = self.actors.get(&child_id) {
            return Ok(r.clone());
        }

        let ctx = TbActorCtx {
            self_id: child_id.clone(),
            system: self.clone(),
        };

        let ms = self.mailbox_settings();
        let (actor_ref, handle) = TbActorMailbox::spawn(child_id.clone(), actor, ctx, ms);

        self.actors.insert(child_id.clone(), actor_ref.clone());
        self.handles.insert(child_id.clone(), handle);

        // Register parent-child relationship.
        self.parent_to_children
            .entry(parent_id.clone())
            .or_default()
            .insert(child_id.clone());
        self.parent_to_children
            .entry(child_id.clone())
            .or_default();

        if let Ok(mut c2p) = self.child_to_parent.write() {
            c2p.insert(child_id.clone(), parent_id.clone());
        }

        debug!("child actor created: {child_id} (parent: {parent_id})");
        Ok(actor_ref)
    }

    fn stop_actor(&self, id: &TbActorId) {
        // Recursively stop children first.
        let children = self.get_children(id);
        for child_id in children {
            self.stop_actor(&child_id);
        }

        // Remove from registry.
        self.actors.remove(id);
        self.parent_to_children.remove(id);

        // Remove from parent's children set.
        if let Ok(c2p) = self.child_to_parent.read() {
            if let Some(parent_id) = c2p.get(id) {
                if let Some(mut children_set) = self.parent_to_children.get_mut(parent_id) {
                    children_set.remove(id);
                }
            }
        }
        if let Ok(mut c2p) = self.child_to_parent.write() {
            c2p.remove(id);
        }

        // Abort the task.
        if let Some((_, handle)) = self.handles.remove(id) {
            handle.abort();
            debug!("actor {id} stopped");
        }
    }

    fn get_children(&self, parent: &TbActorId) -> Vec<TbActorId> {
        self.parent_to_children
            .get(parent)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }
}
