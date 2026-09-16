use cyb_core::{
    GraphSession,
    robot::{host::Host, tasks::*, *},
};
use soma_kernel::agent::{self, Event, Parameters, Provider};
use std::time::{Duration, Instant};
#[test]
fn actual_local_model_crosses_the_captured_neuron_worker_and_durable_result_boundary() {
    let model_path = std::env::var_os("SOMA_TEST_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(soma_kernel::default_model_path);
    // This is an explicit local integration profile. Missing weights are a
    // failure when selected; the test never reports a silent successful skip.
    let model = agent::pin(
        &model_path,
        32,
        Parameters {
            temperature: 0.0,
            ..Default::default()
        },
    )
    .expect("local integration model is required");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("graph");
    let session = GraphSession::open(&path).unwrap();
    let registry = Registry::from_database(session.database()).unwrap();
    let host = Host::new(
        dir.path().into(),
        dir.path().join("robot"),
        "desktop".into(),
    )
    .unwrap();
    host.execute(&registry, &["create", "model-test", &remote::hex(&[1; 32])])
        .unwrap();
    let attachment = registry.robot().unwrap().selected.unwrap();
    let tasks = host
        .task_agent(
            &registry,
            attachment,
            0,
            TaskOptions {
                boot: [2; 32],
                slots: 4,
                instruction_budget: 1_000_000,
            },
        )
        .unwrap();
    let p = &tasks.profile;
    let context = Context {
        subject: p.subject,
        network: p.network,
        attachment: p.attachment,
        binding_revision: p.binding_revision,
        policy: p.policy,
        epoch: p.epoch,
        now: 1,
        soul: [3; 32],
        model,
        tools: vec![],
        workspace: dir.path().to_str().unwrap().into(),
        workspace_revision: [4; 32],
        memory: vec![],
        disclosure: vec![],
        conversation: None,
        goal: None,
        plan: None,
        rounds: 4,
    };
    let id = tasks
        .submit(Input {
            nonce: [5; 32],
            text: "Reply with exactly one word: ready".into(),
            context,
            allowance: 10_000,
            parent: None,
        })
        .unwrap();
    let mut provider = Tools {
        model: Provider::spawn().unwrap(),
    };
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut started = 0;
    let mut observed = 0;
    let mut deltas = String::new();
    let answer;
    loop {
        assert!(
            Instant::now() < deadline,
            "local model exceeded integration deadline"
        );
        while let Some(event) = provider.model.poll() {
            match event {
                Event::Metrics(_, _, tokens, rate) => {
                    assert!(tokens > 0);
                    assert!(rate > 0.0);
                }
                Event::Unknown(_) => panic!("model worker lost its observation"),
                Event::Started(c) => {
                    assert_eq!(c.task, id);
                    started += 1;
                }
                Event::Delta(c, text) => {
                    assert_eq!(c.task, id);
                    deltas.push_str(&text);
                }
                Event::Observed(c, output) => {
                    assert_eq!(c.task, id);
                    tasks
                        .observe(c.task, c.operation, c.attempt, output)
                        .unwrap();
                    observed += 1;
                }
            }
        }
        match tasks.poll(id, &mut provider).unwrap() {
            Progress::Completed(text) => {
                answer = text;
                break;
            }
            Progress::Failed(reason) => panic!("local model task failed: {reason}"),
            Progress::Cancelled => panic!("unexpected cancellation"),
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    assert!(!answer.trim().is_empty());
    assert_eq!(started, 1);
    assert_eq!(observed, 1);
    assert!(!deltas.is_empty());
    let subject = tasks.profile.subject;
    let network = tasks.profile.network;
    drop(provider);
    drop(tasks);
    drop(registry);
    drop(session);
    let reopened = GraphSession::open(&path).unwrap();
    let reader = TaskReader::new(reopened.database(), subject, network).unwrap();
    assert_eq!(reader.task(id).unwrap().answer, Some(answer));
    assert_eq!(reader.observations(id).unwrap().len(), 1);
}
