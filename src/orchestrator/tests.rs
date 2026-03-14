use super::*;

/// Integration test: full spawn → execute → announce lifecycle.
#[tokio::test]
async fn full_lifecycle_spawn_execute_announce() {
    let orch = traits::DefaultOrchestrator::new();
    let parent = session::SessionKey::main("orchestrator");

    // 1. Spawn a subagent.
    let params = spawn::SpawnParams {
        task: "Analyze competitor pricing".into(),
        label: Some("competitor-research".into()),
        agent_id: Some("researcher".into()),
        model: Some("anthropic/claude-haiku-4-5".into()),
        mode: spawn::SpawnMode::Run,
        sandbox: spawn::SandboxMode::Inherit,
        timeout_secs: Some(60),
        expects_completion: true,
        cleanup: spawn::CleanupStrategy::Delete,
        allowed_tools: vec!["web_search".into(), "web_fetch".into()],
    };

    let result = orch.spawn(&parent, params, 0).await;
    assert_eq!(result.status, spawn::SpawnStatus::Accepted);
    let run_id = result.run_id.unwrap();
    let child_key = result.child_session_key.unwrap();
    assert_eq!(child_key.agent_id, "researcher");

    // 2. Verify it's tracked.
    let children = orch.list_children(&parent).await;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].run_id, run_id);

    // 3. Simulate completion.
    orch.registry()
        .mark_ended(
            &run_id,
            registry::RunOutcome {
                success: true,
                summary: Some("Found 3 competitor pricing tiers".into()),
                error: None,
            },
            registry::EndReason::Completed,
            Some("Competitor A: $10/mo, Competitor B: $25/mo, Competitor C: $50/mo".into()),
        )
        .await;

    // 4. Process announce.
    let delivered = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let delivered_clone = delivered.clone();

    announce::process_pending_announces(orch.registry(), |rid, parent_key, msg| {
        let d = delivered_clone.clone();
        async move {
            d.lock().await.push((rid, parent_key, msg));
            Ok(true)
        }
    })
    .await;

    let d = delivered.lock().await;
    assert_eq!(d.len(), 1);
    assert!(d[0].2.contains("competitor-research"));
    assert!(d[0].2.contains("Competitor A"));

    // 5. Verify announced.
    assert!(orch.registry().pending_announces().await.is_empty());
}

/// Test: multiple concurrent subagents with different roles.
#[tokio::test]
async fn multi_role_concurrent_spawns() {
    let orch = traits::DefaultOrchestrator::new();
    let parent = session::SessionKey::main("main");

    let roles = vec![
        ("researcher", "Research market trends"),
        ("architect", "Design API schema"),
        ("engineer", "Implement auth module"),
    ];

    let mut run_ids = Vec::new();
    for (agent, task) in &roles {
        let params = spawn::SpawnParams {
            task: task.to_string(),
            label: Some(agent.to_string()),
            agent_id: Some(agent.to_string()),
            model: None,
            mode: spawn::SpawnMode::Run,
            sandbox: spawn::SandboxMode::Inherit,
            timeout_secs: None,
            expects_completion: true,
            cleanup: spawn::CleanupStrategy::Delete,
            allowed_tools: Vec::new(),
        };
        let result = orch.spawn(&parent, params, 0).await;
        assert_eq!(result.status, spawn::SpawnStatus::Accepted);
        run_ids.push(result.run_id.unwrap());
    }

    // All three should be active.
    let children = orch.list_children(&parent).await;
    assert_eq!(children.len(), 3);
    assert_eq!(orch.registry().active_count_for_parent(&parent).await, 3);

    // Complete one.
    orch.registry()
        .mark_ended(
            &run_ids[0],
            registry::RunOutcome {
                success: true,
                summary: Some("done".into()),
                error: None,
            },
            registry::EndReason::Completed,
            None,
        )
        .await;

    assert_eq!(orch.registry().active_count_for_parent(&parent).await, 2);
}

/// Test: session key parsing and relationships.
#[test]
fn session_key_hierarchy() {
    let parent = session::SessionKey::main("orchestrator");
    let child = session::SessionKey::subagent("researcher");
    let cron = session::SessionKey::cron("daily-report");

    assert_eq!(
        parent.session_type,
        session::SessionType::Main
    );
    assert_eq!(
        child.session_type,
        session::SessionType::Subagent
    );
    assert_eq!(
        cron.session_type,
        session::SessionType::Cron
    );

    // Parse roundtrip.
    let parsed = session::SessionKey::parse(&parent.to_key()).unwrap();
    assert_eq!(parsed.agent_id, "orchestrator");
}
