#[cfg(test)]
mod actor_tests {
    use crate::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    /// Simple counter actor for testing.
    struct CounterActor {
        count: Arc<AtomicU64>,
    }

    #[async_trait]
    impl TbActor for CounterActor {
        async fn process(&mut self, msg: ActorMsg) -> bool {
            match msg {
                ActorMsg::StatsPersistTick => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    true
                }
                _ => false,
            }
        }

        async fn init(&mut self, _ctx: TbActorCtx) -> Result<(), ActorError> {
            Ok(())
        }

        async fn destroy(&mut self, _reason: StopReason) {}
    }

    #[tokio::test]
    async fn test_create_root_actor() {
        let system = TbActorSystem::new(TbActorSystemSettings::default());
        let count = Arc::new(AtomicU64::new(0));
        let count2 = count.clone();

        let id = TbActorId::named("test");
        let actor_ref = system
            .create_root_actor(id.clone(), move || {
                Box::new(CounterActor { count: count2 })
            })
            .unwrap();

        // Send some messages.
        actor_ref.tell(ActorMsg::StatsPersistTick);
        actor_ref.tell(ActorMsg::StatsPersistTick);
        actor_ref.tell(ActorMsg::StatsPersistTick);

        // Give the actor time to process.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert_eq!(count.load(Ordering::SeqCst), 3);
        assert_eq!(system.actor_count(), 1);

        system.shutdown().await;
    }

    #[tokio::test]
    async fn test_parent_child_hierarchy() {
        let system = TbActorSystem::new(TbActorSystemSettings::default());
        let parent_count = Arc::new(AtomicU64::new(0));
        let child_count = Arc::new(AtomicU64::new(0));

        let pc = parent_count.clone();
        let cc = child_count.clone();

        // Parent actor that creates a child on AppInit.
        struct ParentActor {
            count: Arc<AtomicU64>,
            child_count: Arc<AtomicU64>,
            ctx: Option<TbActorCtx>,
        }

        #[async_trait]
        impl TbActor for ParentActor {
            async fn process(&mut self, msg: ActorMsg) -> bool {
                match msg {
                    ActorMsg::AppInit => {
                        self.count.fetch_add(1, Ordering::SeqCst);
                        // Create child actor.
                        if let Some(ctx) = &self.ctx {
                            let cc = self.child_count.clone();
                            let child_id = TbActorId::named("child");
                            let _ = ctx.get_or_create_child(child_id, move || {
                                Box::new(CounterActor { count: cc })
                            });
                        }
                        true
                    }
                    _ => false,
                }
            }

            async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
                self.ctx = Some(ctx);
                Ok(())
            }

            async fn destroy(&mut self, _reason: StopReason) {}
        }

        let parent_id = TbActorId::named("parent");
        let actor_ref = system
            .create_root_actor(parent_id.clone(), move || {
                Box::new(ParentActor {
                    count: pc,
                    child_count: cc,
                    ctx: None,
                })
            })
            .unwrap();

        // Send AppInit → parent creates child.
        actor_ref.tell(ActorMsg::AppInit);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert_eq!(parent_count.load(Ordering::SeqCst), 1);
        assert_eq!(system.actor_count(), 2); // parent + child

        // Send to child directly.
        let child_id = TbActorId::named("child");
        system.tell(&child_id, ActorMsg::StatsPersistTick);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(child_count.load(Ordering::SeqCst), 1);

        // Stop parent → child should also stop.
        system.stop(&parent_id);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(system.actor_count(), 0);
    }

    #[tokio::test]
    async fn test_high_priority_messages() {
        let system = TbActorSystem::new(TbActorSystemSettings::default());
        let count = Arc::new(AtomicU64::new(0));
        let c = count.clone();

        let id = TbActorId::named("hp");
        let actor_ref = system
            .create_root_actor(id, move || Box::new(CounterActor { count: c }))
            .unwrap();

        // High-priority via tell_high_priority.
        actor_ref.tell_high_priority(ActorMsg::StatsPersistTick);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);

        system.shutdown().await;
    }

    #[tokio::test]
    async fn test_actor_id_display() {
        let id1 = TbActorId::entity(uuid::Uuid::nil(), EntityType::Device);
        assert!(format!("{id1}").contains("Device"));

        let id2 = TbActorId::named("APP");
        assert_eq!(format!("{id2}"), "APP");

        let id3 = TbActorId::calculated_field(uuid::Uuid::nil());
        assert!(format!("{id3}").starts_with("CF["));
    }

    #[tokio::test]
    async fn test_broadcast_to_children() {
        let system = TbActorSystem::new(TbActorSystemSettings::default());
        let child1_count = Arc::new(AtomicU64::new(0));
        let child2_count = Arc::new(AtomicU64::new(0));

        struct BroadcastParent {
            c1: Arc<AtomicU64>,
            c2: Arc<AtomicU64>,
            ctx: Option<TbActorCtx>,
        }

        #[async_trait]
        impl TbActor for BroadcastParent {
            async fn process(&mut self, msg: ActorMsg) -> bool {
                match msg {
                    ActorMsg::AppInit => {
                        if let Some(ctx) = &self.ctx {
                            let c1 = self.c1.clone();
                            let _ = ctx.get_or_create_child(TbActorId::named("c1"), move || {
                                Box::new(CounterActor { count: c1 })
                            });
                            let c2 = self.c2.clone();
                            let _ = ctx.get_or_create_child(TbActorId::named("c2"), move || {
                                Box::new(CounterActor { count: c2 })
                            });
                            // Broadcast to children.
                            ctx.broadcast_to_children(
                                &|| ActorMsg::StatsPersistTick,
                                false,
                            );
                        }
                        true
                    }
                    _ => false,
                }
            }

            async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
                self.ctx = Some(ctx);
                Ok(())
            }

            async fn destroy(&mut self, _reason: StopReason) {}
        }

        let c1 = child1_count.clone();
        let c2 = child2_count.clone();
        let parent_id = TbActorId::named("bp");
        let actor_ref = system
            .create_root_actor(parent_id, move || {
                Box::new(BroadcastParent {
                    c1,
                    c2,
                    ctx: None,
                })
            })
            .unwrap();

        actor_ref.tell(ActorMsg::AppInit);
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        assert_eq!(child1_count.load(Ordering::SeqCst), 1);
        assert_eq!(child2_count.load(Ordering::SeqCst), 1);

        system.shutdown().await;
    }
}
