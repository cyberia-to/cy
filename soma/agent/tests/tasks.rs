use cybergraph::application::{Backend, Database};
use neuron_engine::{Action, Authority, Error};
use neuron_node::{Grant, GrantHandle, KeyVault, LocalAuthority, SigningVault};
use soma_agent::*;
use std::sync::{Arc, Mutex};
struct FaultAuthority {
    inner: LocalAuthority<KeyVault>,
    fail: Arc<Mutex<Option<String>>>,
}
impl Authority for FaultAuthority {
    fn authorize(&self, a: &Action<'_>) -> std::result::Result<Vec<u8>, Error> {
        self.inner.authorize(a)
    }
    fn with_current(
        &self,
        a: &Action<'_>,
        c: &mut dyn FnMut() -> std::result::Result<(), Error>,
    ) -> std::result::Result<(), Error> {
        let mut fail = self.fail.lock().unwrap();
        if fail.as_deref() == Some(a.kind) {
            *fail = None;
            return Err(Error::Denied);
        }
        drop(fail);
        self.inner.with_current(a, c)
    }
}
type TestAgent = Agent<FaultAuthority>;
fn open(path: &std::path::Path, boot: u8) -> (TestAgent, GrantHandle) {
    let vault = KeyVault::new(mudra::SigningKey::from_bytes((&[7u8; 32]).into()).unwrap());
    let subject = vault.subject();
    let grant = GrantHandle::new(Grant {
        neuron: subject,
        network: [1; 32],
        policy: [2; 32],
        epoch: 0,
        revision: 0,
        enabled: true,
        acts: [HOST].into(),
        progs: None,
    })
    .unwrap();
    let authority = FaultAuthority {
        inner: LocalAuthority::new(vault, grant.clone()).unwrap(),
        fail: Arc::new(Mutex::new(None)),
    };
    let database = Database::open(path.join("graph"), Backend::Ssd).unwrap();
    let profile = Profile {
        subject,
        network: [1; 32],
        policy: [2; 32],
        epoch: 0,
        attachment: [3; 32],
        binding_revision: 0,
        device: [4; 32],
        worker: [5; 32],
        boot: [boot; 32],
        slots: 4,
        total_steps: 10_000_000,
    };
    (Agent::open(database, authority, profile).unwrap(), grant)
}
fn input(agent: &TestAgent, nonce: u8, path: &std::path::Path) -> Input {
    let p = &agent.profile;
    Input {
        nonce: [nonce; 32],
        text: "read fact then answer".into(),
        allowance: 100_000,
        parent: None,
        context: Context {
            subject: p.subject,
            network: p.network,
            attachment: p.attachment,
            binding_revision: p.binding_revision,
            policy: p.policy,
            epoch: p.epoch,
            now: 1,
            soul: [8; 32],
            model: Model {
                provider: "fixture/1".into(),
                name: "fixture-old".into(),
                revision: [9; 32],
                max_tokens: 64,
                parameters: "{}".into(),
            },
            tools: vec![read_tool(), hash_tool()],
            workspace: path.to_str().unwrap().into(),
            workspace_revision: [10; 32],
            memory: vec![],
            disclosure: vec![],
            conversation: None,
            goal: None,
            plan: None,
            rounds: 32,
        },
    }
}
struct ModelFixture;
impl ModelProvider for ModelFixture {
    fn generate(&mut self, r: &Request) -> AdapterOutcome {
        let decision = if let Some(value) = r.history.iter().find_map(|o| match &o.output {
            Output::Tool { value } => Some(value),
            _ => None,
        }) {
            Decision::Final {
                answer: format!("{}: {value}", r.context.model.name),
            }
        } else {
            Decision::Tool {
                name: read_tool().name,
                schema: read_tool().schema,
                arguments: serde_json::json!({"name":"fact.txt","expected":null}).to_string(),
            }
        };
        AdapterOutcome::Observed(Output::Model { decision })
    }
}
fn drive(a: &TestAgent, id: Id, adapter: &mut impl Adapter) -> Progress {
    for _ in 0..500 {
        match a.poll(id, adapter).unwrap() {
            Progress::Running => {}
            other => return other,
        }
    }
    panic!("bounded task did not reach boundary")
}
struct NoCall;
impl Adapter for NoCall {
    fn call(&mut self, _: &Request) -> AdapterOutcome {
        panic!("physical call forbidden")
    }
}
struct Pause {
    calls: usize,
    phase: Phase,
}
impl Adapter for Pause {
    fn call(&mut self, r: &Request) -> AdapterOutcome {
        self.calls += 1;
        self.phase = r.phase.clone();
        AdapterOutcome::Pending
    }
}
#[test]
fn tool_suspend_restart_exact_observation_completion_and_independent_prog() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fact.txt"), "the sky is green").unwrap();
    let (id, op, attempt, original_input);
    {
        let (a, _) = open(dir.path(), 1);
        original_input = input(&a, 20, dir.path());
        id = a.submit(original_input.clone()).unwrap();
        assert_eq!(a.submit(original_input.clone()).unwrap(), id);
        let mut changed = original_input.clone();
        changed.text.push('!');
        assert!(a.submit(changed).is_err());
        let mut model = Tools {
            model: ModelFixture,
        };
        // Reach the model observation and leave the tool at a later host boundary.
        while a.observations(id).unwrap().is_empty() {
            assert_eq!(a.poll(id, &mut model).unwrap(), Progress::Running);
        }
        let mut pause = Pause {
            calls: 0,
            phase: Phase::Terminal,
        };
        let Progress::Unknown {
            operation,
            attempt: at,
        } = drive(&a, id, &mut pause)
        else {
            panic!("tool not pending")
        };
        assert!(matches!(pause.phase, Phase::Tool { .. }));
        assert_eq!(pause.calls, 1);
        op = operation;
        attempt = at;
        assert!(matches!(
            drive(&a, id, &mut NoCall),
            Progress::Unknown { .. }
        ));
        let other = a.submit(input(&a, 21, dir.path())).unwrap();
        assert_ne!(a.task(other).unwrap().prog, a.task(id).unwrap().prog);
        assert_eq!(
            drive(&a, other, &mut model),
            Progress::Completed("fixture-old: the sky is green".into())
        );
    }
    let (a, _) = open(dir.path(), 2);
    assert_eq!(a.submit(original_input).unwrap(), id);
    assert!(matches!(
        drive(&a, id, &mut NoCall),
        Progress::Unknown { .. }
    ));
    assert!(
        a.observe(
            id,
            [0; 32],
            attempt,
            Output::Tool {
                value: "forged".into()
            }
        )
        .is_err()
    );
    a.observe(
        id,
        op,
        attempt,
        Output::Tool {
            value: "the sky is green".into(),
        },
    )
    .unwrap();
    a.observe(
        id,
        op,
        attempt,
        Output::Tool {
            value: "the sky is green".into(),
        },
    )
    .unwrap();
    assert!(
        a.observe(
            id,
            op,
            attempt,
            Output::Tool {
                value: "different".into()
            }
        )
        .is_err()
    );
    let mut tools = Tools {
        model: ModelFixture,
    };
    assert_eq!(
        drive(&a, id, &mut tools),
        Progress::Completed("fixture-old: the sky is green".into())
    );
    let observations = a.observations(id).unwrap();
    assert_eq!(observations.len(), 3);
    let proposal = a
        .propose_learning(
            id,
            "remember a checked fact".into(),
            vec!["tool receipt reviewed".into()],
        )
        .unwrap();
    assert_ne!(proposal, [0; 32]);
    assert_eq!(a.learning().unwrap().len(), 1);
    assert_eq!(
        a.context(a.task(id).unwrap().control.context).unwrap().soul,
        [8; 32]
    );
}
#[test]
fn context_change_and_steering_preserve_old_result_then_use_admitted_model() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let original = input(&a, 22, dir.path());
    let id = a.submit(original.clone()).unwrap();
    let mut pause = Pause {
        calls: 0,
        phase: Phase::Terminal,
    };
    let Progress::Unknown { operation, attempt } = drive(&a, id, &mut pause) else {
        panic!()
    };
    let mut next = original.context;
    next.model.name = "fixture-new".into();
    next.model.revision = [11; 32];
    a.control(id, 0, Change::Context(next)).unwrap();
    a.control(id, 1, Change::Steer("use the next model".into()))
        .unwrap();
    assert!(a.control(id, 0, Change::Cancel).is_err());
    a.observe(
        id,
        operation,
        attempt,
        Output::Model {
            decision: Decision::Tool {
                name: hash_tool().name,
                schema: hash_tool().schema,
                arguments: "old".into(),
            },
        },
    )
    .unwrap();
    // An old-context tool request is superseded without a physical tool call.
    while a.observations(id).unwrap().len() < 2 {
        assert_eq!(a.poll(id, &mut NoCall).unwrap(), Progress::Running);
    }
    struct Check;
    impl Adapter for Check {
        fn call(&mut self, r: &Request) -> AdapterOutcome {
            assert_eq!(r.context.model.name, "fixture-new");
            assert_eq!(r.steering, ["use the next model"]);
            assert_eq!(r.history[0].attempt.context, r.history[1].attempt.context);
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "changed".into(),
                },
            })
        }
    }
    assert_eq!(
        drive(&a, id, &mut Check),
        Progress::Completed("changed".into())
    );
    assert_eq!(
        a.context(a.observations(id).unwrap()[0].attempt.context)
            .unwrap()
            .model
            .name,
        "fixture-old"
    );
}
#[test]
fn cancellation_unknown_result_revocation_and_no_retry() {
    let dir = tempfile::tempdir().unwrap();
    let (a, grant) = open(dir.path(), 1);
    let id = a.submit(input(&a, 23, dir.path())).unwrap();
    let mut pause = Pause {
        calls: 0,
        phase: Phase::Terminal,
    };
    let Progress::Unknown { operation, attempt } = drive(&a, id, &mut pause) else {
        panic!()
    };
    a.control(id, 0, Change::Cancel).unwrap();
    assert!(matches!(
        drive(&a, id, &mut NoCall),
        Progress::Unknown { .. }
    ));
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision += 1;
    grant.replace(0, revoked.clone()).unwrap();
    // Observation retention does not require a new physical effect grant.
    a.observe(
        id,
        operation,
        attempt,
        Output::Model {
            decision: Decision::Final {
                answer: "late".into(),
            },
        },
    )
    .unwrap();
    assert!(a.poll(id, &mut NoCall).is_err());
    assert_eq!(a.observations(id).unwrap().len(), 1);
    revoked.enabled = true;
    revoked.revision += 1;
    grant.replace(1, revoked).unwrap();
    assert_eq!(drive(&a, id, &mut NoCall), Progress::Cancelled);
    assert_eq!(pause.calls, 1);
}
struct Delegator {
    policy: Join,
    fail_second: bool,
}
impl Adapter for Delegator {
    fn call(&mut self, r: &Request) -> AdapterOutcome {
        if r.input == "child one" || r.input == "child two" {
            if self.fail_second && r.input == "child two" {
                return AdapterOutcome::Observed(Output::Failed {
                    reason: "fixture child failure".into(),
                });
            }
            return AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: r.input.clone(),
                },
            });
        }
        if r.history
            .iter()
            .any(|o| matches!(o.output, Output::Joined { .. }))
        {
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "joined".into(),
                },
            })
        } else {
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Delegate {
                    children: vec![
                        Child {
                            input: "child one".into(),
                            allowance: 10_000,
                            disclosure: vec![],
                        },
                        Child {
                            input: "child two".into(),
                            allowance: 10_000,
                            disclosure: vec![],
                        },
                    ],
                    join: self.policy.clone(),
                },
            })
        }
    }
}
#[test]
fn all_join_policies_keep_lineage_budget_and_recover_children() {
    for policy in [Join::AllRequired, Join::FirstSuccess, Join::BestEffort] {
        let dir = tempfile::tempdir().unwrap();
        let parent;
        {
            let (a, _) = open(dir.path(), 1);
            parent = a.submit(input(&a, 30, dir.path())).unwrap();
            let mut adapter = Delegator {
                policy: policy.clone(),
                fail_second: policy == Join::BestEffort,
            };
            assert_eq!(drive(&a, parent, &mut adapter), Progress::Waiting);
            let task = a.task(parent).unwrap();
            assert_eq!(task.children.len(), 2);
            let view = a.engine.inspect(a.profile.subject).unwrap();
            let job = &view.state.invocations[&parent];
            assert_eq!(job.delegated, 20_000);
            assert_eq!(job.children.len(), 2);
            for child in &task.children {
                assert_eq!(view.state.invocations[child].parent, Some(parent));
            }
        }
        let (a, _) = open(dir.path(), 2);
        let task = a.task(parent).unwrap();
        let mut adapter = Delegator {
            policy: policy.clone(),
            fail_second: policy == Join::BestEffort,
        };
        assert!(matches!(
            drive(&a, task.children[0], &mut adapter),
            Progress::Completed(_)
        ));
        if policy == Join::FirstSuccess {
            assert_eq!(drive(&a, parent, &mut adapter), Progress::Waiting);
            assert_eq!(
                drive(&a, task.children[1], &mut NoCall),
                Progress::Cancelled
            );
        } else {
            drive(&a, task.children[1], &mut adapter);
        }
        assert_eq!(
            drive(&a, parent, &mut adapter),
            Progress::Completed("joined".into())
        );
        let view = a.engine.inspect(a.profile.subject).unwrap();
        assert_eq!(
            view.state.invocations[&parent].delegated,
            task.children
                .iter()
                .map(|id| {
                    let j = &view.state.invocations[id];
                    j.charged + j.delegated
                })
                .sum::<u64>()
        );
        assert_eq!(view.state.held, 0);
        assert_eq!(a.tasks().unwrap().len(), 3);
    }
}
#[test]
fn required_join_failure_and_parent_cancel_stop_remaining_children() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let parent = a.submit(input(&a, 31, dir.path())).unwrap();
    let mut adapter = Delegator {
        policy: Join::AllRequired,
        fail_second: true,
    };
    assert_eq!(drive(&a, parent, &mut adapter), Progress::Waiting);
    let task = a.task(parent).unwrap();
    assert!(matches!(
        drive(&a, task.children[1], &mut adapter),
        Progress::Failed(_)
    ));
    assert_eq!(drive(&a, parent, &mut adapter), Progress::Waiting);
    assert!(matches!(
        drive(&a, parent, &mut adapter),
        Progress::Failed(_)
    ));
    assert_eq!(
        drive(&a, task.children[0], &mut NoCall),
        Progress::Cancelled
    );
    let other = a.submit(input(&a, 32, dir.path())).unwrap();
    assert_eq!(drive(&a, other, &mut adapter), Progress::Waiting);
    a.control(other, 0, Change::Cancel).unwrap();
    assert_eq!(drive(&a, other, &mut NoCall), Progress::Cancelled);
    let view = a.engine.inspect(a.profile.subject).unwrap();
    assert_eq!(view.state.held, 0);
}
#[test]
fn schedule_restart_one_occurrence_exact_admission_then_disable() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fact.txt"), "scheduled").unwrap();
    let first;
    {
        let (a, _) = open(dir.path(), 1);
        a.schedule(Schedule {
            id: [40; 32],
            due: 100,
            interval: Some(10),
            remaining: 2,
            occurrence: 0,
            enabled: true,
            input: input(&a, 41, dir.path()),
            receipts: vec![],
        })
        .unwrap();
        assert_eq!(a.due(99).unwrap(), None);
        first = a.due(100).unwrap().unwrap();
        assert_eq!(a.due(100).unwrap(), None);
    }
    let (a, _) = open(dir.path(), 2);
    assert_eq!(a.schedules().unwrap()[0].receipts, [first]);
    a.enable_schedule([40; 32], 1, false).unwrap();
    assert_eq!(a.due(1000).unwrap(), None);
    a.enable_schedule([40; 32], 1, true).unwrap();
    let second = a.due(1000).unwrap().unwrap();
    assert_ne!(first, second);
    assert_eq!(a.due(1000).unwrap(), None);
    let mut adapter = Tools {
        model: ModelFixture,
    };
    assert_eq!(
        drive(&a, first, &mut adapter),
        Progress::Completed("fixture-old: scheduled".into())
    );
    assert_eq!(
        drive(&a, second, &mut adapter),
        Progress::Completed("fixture-old: scheduled".into())
    );
    assert_eq!(a.tasks().unwrap().len(), 2);
    assert_eq!(a.schedules().unwrap()[0].occurrence, 2);
}
#[test]
fn tool_schema_scope_output_bounds_and_symlinks_fail_as_observed_results() {
    struct BadTool;
    impl Adapter for BadTool {
        fn call(&mut self, _: &Request) -> AdapterOutcome {
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Tool {
                    name: "unknown".into(),
                    schema: [0; 32],
                    arguments: "anything".into(),
                },
            })
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let id = a.submit(input(&a, 50, dir.path())).unwrap();
    assert!(matches!(drive(&a, id, &mut BadTool), Progress::Failed(_)));
    assert_eq!(a.observations(id).unwrap().len(), 1);
    let mut wrong = input(&a, 51, dir.path());
    wrong.context.network = [0; 32];
    assert!(a.submit(wrong).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", dir.path().join("fact.txt")).unwrap();
        let id = a.submit(input(&a, 52, dir.path())).unwrap();
        assert!(matches!(
            drive(
                &a,
                id,
                &mut Tools {
                    model: ModelFixture
                }
            ),
            Progress::Failed(_)
        ));
    }
}

#[test]
fn admission_and_schedule_receipt_faults_recover_exactly_once() {
    for kind in ["admit", "soma/admitted", "soma/occurrence"] {
        let dir = tempfile::tempdir().unwrap();
        let schedule = {
            let (a, _) = open(dir.path(), 1);
            let schedule = Schedule {
                id: [60; 32],
                due: 100,
                interval: None,
                remaining: 1,
                occurrence: 0,
                enabled: true,
                input: input(&a, 61, dir.path()),
                receipts: vec![],
            };
            a.schedule(schedule.clone()).unwrap();
            *a.engine.authority.fail.lock().unwrap() = Some(kind.into());
            assert!(a.due(100).is_err(), "fault {kind} did not fire");
            let view = a.engine.inspect(a.profile.subject).unwrap();
            assert_eq!(view.state.invocations.len(), usize::from(kind != "admit"));
            schedule
        };
        let (a, _) = open(dir.path(), 2);
        let id = a.due(100).unwrap().unwrap();
        assert_eq!(a.due(100).unwrap(), None);
        assert_eq!(a.tasks().unwrap().len(), 1);
        assert_eq!(a.schedules().unwrap()[0].receipts, [id]);
        let view = a.engine.inspect(a.profile.subject).unwrap();
        assert_eq!(view.state.invocations.len(), 1);
        assert_eq!(view.state.held, schedule.input.allowance);
        a.control(id, 0, Change::Cancel).unwrap();
        assert_eq!(drive(&a, id, &mut NoCall), Progress::Cancelled);
    }
}
#[test]
fn observation_before_engine_commit_survives_restart_without_model_reexecution() {
    let dir = tempfile::tempdir().unwrap();
    let id;
    struct Once(usize);
    impl Adapter for Once {
        fn call(&mut self, _: &Request) -> AdapterOutcome {
            self.0 += 1;
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "already observed".into(),
                },
            })
        }
    }
    {
        let (a, _) = open(dir.path(), 1);
        id = a.submit(input(&a, 62, dir.path())).unwrap();
        *a.engine.authority.fail.lock().unwrap() = Some("reconcile".into());
        let mut adapter = Once(0);
        for _ in 0..100 {
            if a.poll(id, &mut adapter).is_err() {
                break;
            }
        }
        assert_eq!(adapter.0, 1);
        assert_eq!(a.observations(id).unwrap().len(), 1);
        let view = a.engine.inspect(a.profile.subject).unwrap();
        assert_eq!(
            view.state.invocations[&id].pending.as_ref().unwrap().stage,
            1
        );
    }
    let (a, _) = open(dir.path(), 2);
    assert_eq!(
        drive(&a, id, &mut NoCall),
        Progress::Completed("already observed".into())
    );
}
#[test]
fn adapter_panic_is_unknown_and_does_not_poison_other_work() {
    struct Panic;
    impl Adapter for Panic {
        fn call(&mut self, _: &Request) -> AdapterOutcome {
            panic!("fixture adapter panic")
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let id = a.submit(input(&a, 63, dir.path())).unwrap();
    assert!(matches!(
        drive(&a, id, &mut Panic),
        Progress::Unknown { .. }
    ));
    assert!(matches!(
        drive(&a, id, &mut NoCall),
        Progress::Unknown { .. }
    ));
    let other = a.submit(input(&a, 64, dir.path())).unwrap();
    a.control(other, 0, Change::Cancel).unwrap();
    assert_eq!(drive(&a, other, &mut NoCall), Progress::Cancelled);
}
#[test]
fn steering_during_pending_final_requires_a_new_model_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let id = a.submit(input(&a, 65, dir.path())).unwrap();
    let mut pause = Pause {
        calls: 0,
        phase: Phase::Terminal,
    };
    let Progress::Unknown { operation, attempt } = drive(&a, id, &mut pause) else {
        panic!()
    };
    a.control(id, 0, Change::Steer("include this correction".into()))
        .unwrap();
    a.observe(
        id,
        operation,
        attempt,
        Output::Model {
            decision: Decision::Final {
                answer: "old answer".into(),
            },
        },
    )
    .unwrap();
    struct Correct;
    impl Adapter for Correct {
        fn call(&mut self, r: &Request) -> AdapterOutcome {
            assert_eq!(r.steering, ["include this correction"]);
            assert_eq!(r.history.len(), 1);
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "corrected answer".into(),
                },
            })
        }
    }
    assert_eq!(
        drive(&a, id, &mut Correct),
        Progress::Completed("corrected answer".into())
    );
    // Historical receipts remain idempotent after a later operation completes.
    a.observe(
        id,
        operation,
        attempt,
        Output::Model {
            decision: Decision::Final {
                answer: "old answer".into(),
            },
        },
    )
    .unwrap();
}
#[test]
fn memory_is_pinned_and_disclosure_is_checked_before_provider_or_child_admission() {
    use cybergraph::{
        application::{Head, Proposal},
        content::{Codec, Content},
    };
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let content = Content::new(Codec::Blob, b"remembered scoped fact".to_vec()).unwrap();
    let id = content.id();
    a.engine
        .graph
        .0
        .commit(
            &Proposal {
                namespace: [70; 32],
                request: [71; 32],
                expected: None,
                head: Head {
                    index: 0,
                    commit: id,
                },
                content: vec![content],
                required: vec![],
                claims: vec![],
            },
            |_| Ok(()),
        )
        .unwrap();
    let mut task_input = input(&a, 72, dir.path());
    task_input.context.tools = vec![read_tool()];
    task_input.context.memory = vec![Memory {
        scope: "local/facts".into(),
        head: id,
        content: id,
    }];
    assert!(a.submit(task_input.clone()).is_err());
    assert_eq!(a.tasks().unwrap().len(), 0);
    task_input.context.disclosure = vec!["local/facts".into()];
    let parent = a.submit(task_input.clone()).unwrap();
    let mut child = input(&a, 73, dir.path());
    child.parent = Some(parent);
    child.context.disclosure = vec!["private/unshared".into()];
    assert!(a.submit(child.clone()).is_err());
    child.context.disclosure.clear();
    assert!(a.submit(child).is_err());
    struct MemoryCheck;
    impl Adapter for MemoryCheck {
        fn call(&mut self, r: &Request) -> AdapterOutcome {
            assert_eq!(r.memory.len(), 1);
            assert_eq!(r.memory[0].0.scope, "local/facts");
            assert_eq!(r.memory[0].1, "remembered scoped fact");
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "remembered".into(),
                },
            })
        }
    }
    assert_eq!(
        drive(&a, parent, &mut MemoryCheck),
        Progress::Completed("remembered".into())
    );
    assert_eq!(a.observations(parent).unwrap().len(), 1);
}
#[test]
fn common_tree_driver_completes_children_and_cancel_precedes_child_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let root = a.submit(input(&a, 80, dir.path())).unwrap();
    let mut adapter = Delegator {
        policy: Join::FirstSuccess,
        fail_second: false,
    };
    let mut finished = false;
    for _ in 0..500 {
        if a.poll_tree(root, &mut adapter).unwrap() == Progress::Completed("joined".into()) {
            finished = true;
            break;
        }
    }
    assert!(finished);
    assert_eq!(a.task(root).unwrap().children.len(), 2);
    let other = a.submit(input(&a, 81, dir.path())).unwrap();
    let mut pause = Pause {
        calls: 0,
        phase: Phase::Terminal,
    };
    assert!(matches!(
        drive(&a, other, &mut pause),
        Progress::Unknown { .. }
    ));
    let mut child_input = input(&a, 82, dir.path());
    child_input.parent = Some(other);
    child_input.allowance = 10_000;
    let child = a.submit(child_input).unwrap();
    a.control(other, 0, Change::Cancel).unwrap();
    assert!(matches!(
        a.poll_tree(other, &mut NoCall).unwrap(),
        Progress::Unknown { .. }
    ));
    assert!(a.task(child).unwrap().control.cancelled);
    assert_eq!(
        a.engine
            .inspect(a.profile.subject)
            .unwrap()
            .state
            .invocations[&child]
            .status
            .name(),
        "cancelled"
    );
}
#[test]
fn terminal_application_observation_does_not_accept_discarded_steering() {
    let dir = tempfile::tempdir().unwrap();
    let (a, _) = open(dir.path(), 1);
    let id = a.submit(input(&a, 83, dir.path())).unwrap();
    struct Final;
    impl Adapter for Final {
        fn call(&mut self, _: &Request) -> AdapterOutcome {
            AdapterOutcome::Observed(Output::Model {
                decision: Decision::Final {
                    answer: "done".into(),
                },
            })
        }
    }
    while a.task(id).unwrap().answer.is_none() {
        a.poll(id, &mut Final).unwrap();
    }
    assert!(
        a.control(id, 0, Change::Steer("too late for this decision".into()))
            .is_err()
    );
    assert_eq!(
        drive(&a, id, &mut NoCall),
        Progress::Completed("done".into())
    );
}
#[test]
fn child_receipt_missing_from_application_recovers_after_parent_cancel_without_readmission() {
    let dir = tempfile::tempdir().unwrap();
    let root;
    let child_nonce = [84; 32];
    {
        let (a, _) = open(dir.path(), 1);
        root = a.submit(input(&a, 85, dir.path())).unwrap();
        let mut child = input(&a, 84, dir.path());
        child.parent = Some(root);
        child.allowance = 10_000;
        *a.engine.authority.fail.lock().unwrap() = Some("soma/admitted".into());
        assert!(a.submit(child).is_err());
        assert_eq!(
            a.engine
                .inspect(a.profile.subject)
                .unwrap()
                .state
                .invocations[&root]
                .children
                .len(),
            1
        );
        assert!(a.task(root).unwrap().children.is_empty());
        a.control(root, 0, Change::Cancel).unwrap();
    }
    let (a, _) = open(dir.path(), 2);
    let recovered = a.recover_admissions().unwrap();
    assert_eq!(recovered.len(), 1);
    let child = recovered[0];
    assert_eq!(a.task(child).unwrap().admission, child_nonce);
    assert!(a.task(child).unwrap().control.cancelled);
    assert_eq!(a.task(root).unwrap().children, [child]);
    assert_eq!(a.poll_tree(root, &mut NoCall).unwrap(), Progress::Cancelled);
    let view = a.engine.inspect(a.profile.subject).unwrap();
    assert_eq!(view.state.invocations.len(), 2);
    assert_eq!(view.state.held, 0);
}
