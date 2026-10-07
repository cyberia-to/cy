//! Durable task strategy over the existing neuron/Rune machine and worker ABI.
//! The host supplies custody and adapters. Tasks and progs never acquire keys.
mod adapters;
mod controls;
mod store;
mod types;
pub use adapters::*;
pub use controls::{Change, LearningProposal};
use cybergraph::{
    application::{Database, Head},
    content::Content,
};
pub use neuron_engine::Authority;
use neuron_engine::{Action, Admission, Dispatch, JobProgress, Neuron, ProgramConfig};
use neuron_model::execution::Status;
use neuron_node::{EffectOutcome, Graph, LocalWorker, Rune};
use store::{Prepared, State, Store, err};
pub use store::{derive, hash, hex};
pub use types::*;

pub const TASK_PROGRAM: &str = r#"
let task = event;
loop {
    let reply = host(task);
    if reply == 0 { 0 } else { rebind task = reply; continue }
}
"#;
#[derive(Clone)]
pub struct Profile {
    pub subject: Id,
    pub network: Id,
    pub policy: Id,
    pub epoch: u64,
    pub attachment: Id,
    pub binding_revision: u64,
    pub device: Id,
    pub worker: Id,
    pub boot: Id,
    pub slots: usize,
    pub total_steps: u64,
}
pub struct Agent<A: Authority> {
    pub engine: Neuron<Graph, Rune, A>,
    worker: LocalWorker,
    store: Store,
    pub profile: Profile,
    tree_cursor: std::sync::atomic::AtomicUsize,
}
fn handle(id: Id) -> Result<Vec<u8>> {
    let mut words = [0; 4];
    for (i, bytes) in id.chunks_exact(8).enumerate() {
        words[i] = u64::from_le_bytes(bytes.try_into().unwrap());
    }
    neuron_rune::value(&format!(
        "[[{} {}] [{} {}]]",
        words[0], words[1], words[2], words[3]
    ))
    .map_err(err)
}
fn zero() -> Vec<u8> {
    neuron_rune::value("0").expect("literal zero")
}
impl<A: Authority> Agent<A> {
    /// Explicit local worker placement. Opening a new body generation fences
    /// the old worker; it does not retry its unresolved effects.
    pub fn open(database: Database, authority: A, mut profile: Profile) -> Result<Self> {
        if profile.slots == 0 || profile.slots > 64 || profile.total_steps == 0 {
            return Err("invalid task capacity".into());
        }
        let engine =
            Neuron::with_authority(Graph::from_database(database.clone()), Rune, authority);
        let view = match engine.inspect(profile.subject) {
            Ok(view) => view,
            Err(neuron_engine::Error::Missing) => {
                engine
                    .activate(
                        profile.subject,
                        profile.network,
                        profile.policy,
                        profile.total_steps,
                    )
                    .map_err(err)?;
                engine.inspect(profile.subject).map_err(err)?
            }
            Err(e) => return Err(err(e)),
        };
        if view.state.network != profile.network
            || view.state.policy != profile.policy
            || view.state.epoch != profile.epoch
        {
            return Err("runtime authority profile mismatch".into());
        }
        profile.total_steps = view.state.limit;
        let worker = LocalWorker::bind(
            &engine,
            profile.subject,
            view.state.writer_generation,
            LocalWorker::descriptor(
                profile.network,
                profile.worker,
                profile.device,
                profile.boot,
                vec![HOST],
                1024,
            ),
        )
        .map_err(err)?;
        let store = Store::new(database, profile.subject, profile.network)?;
        let agent = Self {
            engine,
            worker,
            store,
            profile,
            tree_cursor: std::sync::atomic::AtomicUsize::new(0),
        };
        {
            let _guard = agent
                .store
                .lock
                .lock()
                .map_err(|_| "Soma coordinator poisoned")?;
            let (mut head, mut state) = agent.store.load()?;
            if !state.slots.is_empty() && state.slots.len() > agent.profile.slots {
                return Err("configured task pool smaller than persisted pool".into());
            }
            for slot in state.slots.len()..agent.profile.slots {
                let prog = agent
                    .engine
                    .install(
                        agent.profile.subject,
                        derive(
                            "soma/prog-slot/1",
                            &[&agent.profile.network, &(slot as u64).to_le_bytes()],
                        ),
                        TASK_PROGRAM.as_bytes().to_vec(),
                        zero(),
                        ProgramConfig {
                            step_limit: 1_000_000,
                            max_inflight: 1,
                            allowed_acts: vec![HOST],
                        },
                    )
                    .map_err(err)?;
                state.slots.push(prog);
                head = Some(agent.publish(head, &state, vec![], "soma/slots", Some(prog), None)?);
            }
        }
        Ok(agent)
    }
    fn publish(
        &self,
        head: Option<Head>,
        state: &State,
        content: Vec<Content>,
        kind: &str,
        prog: Option<Id>,
        task: Option<Id>,
    ) -> Result<Head> {
        let statement = self.store.content("soma/state/1", state)?.id();
        let p = &self.profile;
        let action = Action {
            neuron: p.subject,
            network: p.network,
            policy: p.policy,
            epoch: p.epoch,
            prog,
            invocation: task,
            act: Some(HOST),
            kind,
            statement,
            allowed_acts: &[HOST],
        };
        let mut content = Some(content);
        let mut result = None;
        self.engine
            .authority
            .with_current(&action, &mut || {
                result = Some(self.store.save(
                    head,
                    state,
                    content.take().ok_or(neuron_engine::Error::Conflict)?,
                ));
                Ok(())
            })
            .map_err(err)?;
        result.ok_or("Soma publication was not called")?
    }
    fn context_scope(&self, context: &Context) -> Result<()> {
        context.validate()?;
        let p = &self.profile;
        if context.subject != p.subject
            || context.network != p.network
            || context.policy != p.policy
            || context.epoch != p.epoch
            || context.attachment != p.attachment
            || context.binding_revision != p.binding_revision
        {
            return Err("context does not match captured authority".into());
        }
        Ok(())
    }
    pub fn submit(&self, input: Input) -> Result<Id> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        self.submit_locked(input)
    }
    fn submit_locked(&self, input: Input) -> Result<Id> {
        text(&input.text)?;
        let (mut head, mut state) = self.store.load()?;
        let input_content = self.store.content("soma/input/1", &input)?;
        let input_id = input_content.id();
        let key = hex(&input.nonce);
        if let Some(prepared) = state.prepared.get(&key) {
            if prepared.input != input_id {
                return Err("admission nonce conflict".into());
            }
            if let Some(task) = prepared.task {
                return Ok(task);
            }
        }
        self.context_scope(&input.context)?;
        if input.allowance == 0 || input.allowance > 1_000_000 {
            return Err("task step allowance".into());
        }
        let recovering = state.prepared.contains_key(&key)
            && self
                .engine
                .lookup_admission(
                    self.profile.subject,
                    derive("soma/admission/1", &[&input.nonce]),
                )
                .map_err(err)?
                .is_some();
        if let Some(parent_id) = input.parent.filter(|_| !recovering) {
            let parent = state
                .tasks
                .get(&hex(&parent_id))
                .ok_or("missing Soma parent")?;
            if parent.control.cancelled {
                return Err("parent cancelled".into());
            }
            let context = self.context(parent.control.context)?;
            if input.context.workspace != context.workspace
                || input.context.workspace_revision != context.workspace_revision
                || input.context.soul != context.soul
                || input.context.model != context.model
                || input.context.rounds > context.rounds
                || input
                    .context
                    .tools
                    .iter()
                    .any(|tool| !context.tools.contains(tool))
                || input
                    .context
                    .disclosure
                    .iter()
                    .any(|scope| !context.disclosure.contains(scope))
                || input
                    .context
                    .memory
                    .iter()
                    .any(|memory| !context.memory.contains(memory))
            {
                return Err("child context is not a parent subset".into());
            }
        }
        if !state.prepared.contains_key(&key) {
            let view = self.engine.inspect(self.profile.subject).map_err(err)?;
            let prog = state
                .slots
                .iter()
                .find(|prog| {
                    !view
                        .state
                        .invocations
                        .values()
                        .any(|job| job.prog == **prog && !job.status.terminal())
                        && !state
                            .prepared
                            .values()
                            .any(|p| p.prog == **prog && p.task.is_none())
                })
                .copied()
                .ok_or("task pool occupied")?;
            let context = self.store.content("soma/context/1", &input.context)?;
            // Read memory now: missing, wrong codec or undisclosed sources cannot
            // become an ambiguous provider failure after dispatch.
            self.memory(&input.context)?;
            let step = self.store.content(
                "soma/step/1",
                &Step {
                    admission: input.nonce,
                    previous: None,
                    context: context.id(),
                    round: 0,
                    control: 0,
                    phase: Phase::Model,
                },
            )?;
            state.prepared.insert(
                key.clone(),
                Prepared {
                    input: input_id,
                    prog,
                    step: step.id(),
                    task: None,
                },
            );
            head = Some(self.publish(
                head,
                &state,
                vec![input_content, context, step],
                "soma/admit",
                Some(prog),
                None,
            )?);
        }
        let prepared = state.prepared[&key].clone();
        let step: Step = self.store.read(prepared.step, "soma/step/1")?;
        let receipt = self
            .engine
            .submit(
                self.profile.subject,
                Admission {
                    prog: prepared.prog,
                    nonce: derive("soma/admission/1", &[&input.nonce]),
                    input: handle(prepared.step)?,
                    context: Some(step.context),
                    parent: input.parent,
                    allowance: input.allowance,
                },
            )
            .map_err(err)?;
        let mut parent_cancelled = false;
        if let Some(parent_id) = input.parent {
            let parent = state
                .tasks
                .get_mut(&hex(&parent_id))
                .ok_or("missing parent receipt")?;
            parent_cancelled = parent.control.cancelled;
            if !parent.children.contains(&receipt.invocation) {
                parent.children.push(receipt.invocation);
            }
        }
        let task = Task {
            id: receipt.invocation,
            prog: prepared.prog,
            admission: input.nonce,
            input: input_id,
            step: prepared.step,
            control: Control {
                revision: 0,
                context: step.context,
                steering: vec![],
                cancelled: parent_cancelled,
            },
            pending: None,
            observations: vec![],
            children: vec![],
            answer: None,
            failure: None,
        };
        state.tasks.insert(hex(&task.id), task);
        state.prepared.get_mut(&key).unwrap().task = Some(receipt.invocation);
        self.publish(
            head,
            &state,
            vec![],
            "soma/admitted",
            Some(prepared.prog),
            Some(receipt.invocation),
        )?;
        Ok(receipt.invocation)
    }
    /// Prepared admissions are durable even if the engine or receipt write
    /// failed. Re-admission uses their exact captured context and nonce.
    pub fn recover_admissions(&self) -> Result<Vec<Id>> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let (_, state) = self.store.load()?;
        state
            .prepared
            .values()
            .filter(|p| p.task.is_none())
            .map(|p| self.submit_locked(self.store.read(p.input, "soma/input/1")?))
            .collect()
    }
    pub fn task(&self, id: Id) -> Result<Task> {
        self.store
            .load()?
            .1
            .tasks
            .remove(&hex(&id))
            .ok_or("missing Soma task".into())
    }
    pub fn tasks(&self) -> Result<Vec<Task>> {
        Ok(self.store.load()?.1.tasks.into_values().collect())
    }
    pub fn context(&self, id: Id) -> Result<Context> {
        self.store.read(id, "soma/context/1")
    }
    pub fn observations(&self, id: Id) -> Result<Vec<Observed>> {
        self.task(id)?
            .observations
            .iter()
            .map(|id| self.store.read(*id, "soma/observation/1"))
            .collect()
    }
    fn memory(&self, context: &Context) -> Result<Vec<(Memory, String)>> {
        let mut total = 0;
        context
            .memory
            .iter()
            .map(|source| {
                if !context.disclosure.contains(&source.scope) {
                    return Err("memory scope denied".into());
                }
                // Both the scoped observation and exact content must still exist.
                self.store
                    .graph
                    .get(&source.head)
                    .map_err(err)?
                    .ok_or("missing memory source head")?;
                let content = self
                    .store
                    .graph
                    .get(&source.content)
                    .map_err(err)?
                    .ok_or("missing memory content")?;
                total += content.bytes().len();
                if content.codec() != cybergraph::content::Codec::Blob || total > MAX_TEXT {
                    return Err("memory bounds/codec".into());
                }
                Ok((
                    source.clone(),
                    std::str::from_utf8(content.bytes())
                        .map_err(err)?
                        .to_owned(),
                ))
            })
            .collect()
    }
    /// At most one bounded Rune slice or one adapter handoff. A host can drive
    /// other tasks while this one awaits a provider, tool or child result.
    pub fn poll(&self, id: Id, adapter: &mut impl Adapter) -> Result<Progress> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        self.poll_locked(id, adapter)
    }
    /// Body-independent tree scheduling: parent control first, one descendant
    /// slice in round-robin order, then parent adoption. The cursor is a local
    /// fairness hint; durable task state and budgets remain authoritative.
    pub fn poll_tree(&self, root: Id, adapter: &mut impl Adapter) -> Result<Progress> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let progress = self.poll_locked(root, adapter)?;
        if matches!(
            progress,
            Progress::Completed(_) | Progress::Cancelled | Progress::Failed(_)
        ) {
            return Ok(progress);
        }
        let (_, state) = self.store.load()?;
        let mut seen = std::collections::BTreeSet::from([root]);
        let mut children = state
            .tasks
            .get(&hex(&root))
            .ok_or("missing root task")?
            .children
            .clone();
        let mut index = 0;
        while index < children.len() {
            if children.len() > MAX_TASKS || !seen.insert(children[index]) {
                return Err("task tree bound/cycle".into());
            }
            children.extend(
                state
                    .tasks
                    .get(&hex(&children[index]))
                    .ok_or("missing descendant")?
                    .children
                    .iter()
                    .copied(),
            );
            index += 1;
        }
        if children.is_empty() {
            return Ok(progress);
        }
        let index = self
            .tree_cursor
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % children.len();
        self.poll_locked(children[index], adapter)?;
        self.poll_locked(root, adapter)
    }
    fn poll_locked(&self, id: Id, adapter: &mut impl Adapter) -> Result<Progress> {
        let mut task = self.task(id)?;
        let view = self.engine.inspect(self.profile.subject).map_err(err)?;
        let job = view
            .state
            .invocations
            .get(&id)
            .ok_or("missing task invocation")?;
        if job.prog != task.prog {
            return Err("task program mismatch".into());
        }
        if !job.status.terminal() && !task.control.cancelled {
            let input: Input = self.store.read(task.input, "soma/input/1")?;
            if let Some(parent) = input.parent {
                if self.task(parent)?.control.cancelled {
                    self.mark_cancelled(id)?;
                    task = self.task(id)?;
                }
            }
        }
        if task.control.cancelled && !job.status.terminal() {
            let children_ready = self.cancel_children(&task)?;
            if let Some(p) = &job.pending {
                if p.stage == 1 {
                    if let Some(observed) = self.observations(id)?.into_iter().find(|o| {
                        o.attempt.operation == p.id && Some(o.attempt.attempt) == p.attempt
                    }) {
                        self.reconcile(id, &observed)?;
                        return Ok(Progress::Running);
                    }
                    return Ok(Progress::Unknown {
                        operation: p.id,
                        attempt: p.attempt.ok_or("missing attempt")?,
                    });
                }
            }
            if !children_ready {
                return Ok(Progress::Waiting);
            }
            self.engine
                .cancel(self.profile.subject, id, "Soma control cancellation".into())
                .map_err(err)?;
            return Ok(Progress::Cancelled);
        }
        match self
            .engine
            .tick_invocation(self.profile.subject, id)
            .map_err(err)?
        {
            JobProgress::Finished { status, .. } => match status {
                Status::Completed => task
                    .answer
                    .map(Progress::Completed)
                    .ok_or("engine completed without application terminal result".into()),
                Status::Cancelled => Ok(Progress::Cancelled),
                other => Ok(Progress::Failed(
                    task.failure.unwrap_or_else(|| other.name().into()),
                )),
            },
            JobProgress::Yielded { .. } => Ok(Progress::Running),
            JobProgress::Idle | JobProgress::Event { .. } => Ok(Progress::Waiting),
            JobProgress::Unknown {
                operation, attempt, ..
            } => {
                if let Some(observed) = self
                    .observations(id)?
                    .into_iter()
                    .find(|o| o.attempt.operation == operation && o.attempt.attempt == attempt)
                {
                    self.reconcile(id, &observed)?;
                    Ok(Progress::Running)
                } else {
                    Ok(Progress::Unknown { operation, attempt })
                }
            }
            JobProgress::Awaiting { tag, .. } => {
                if tag != HOST {
                    return Err("unexpected task host act".into());
                }
                let step: Step = self.store.read(task.step, "soma/step/1")?;
                let joined = if matches!(step.phase, Phase::Join { .. }) {
                    match self.join_locked(&task, &step)? {
                        Some(output) => Some(output),
                        None => return Ok(Progress::Waiting),
                    }
                } else {
                    None
                };
                let task = self.task(id)?;
                let token = self
                    .worker
                    .prepare(&self.engine, self.profile.subject, id)
                    .map_err(err)?;
                let mut callback_error = None;
                let execution = self
                    .worker
                    .execute(&self.engine, token, |dispatch| {
                        match self.call(&task, &step, dispatch, adapter, joined) {
                            Ok(outcome) => outcome,
                            Err(e) => {
                                callback_error = Some(e);
                                EffectOutcome::Unknown("Soma observation persistence failed".into())
                            }
                        }
                    })
                    .map_err(err)?;
                if let Some(e) = callback_error {
                    return Err(e);
                }
                if let Some(reconciliation) = execution.reconciliation {
                    reconciliation.map_err(err)?;
                    Ok(Progress::Running)
                } else {
                    Ok(Progress::Unknown {
                        operation: execution.dispatch.operation,
                        attempt: execution.dispatch.attempt,
                    })
                }
            }
        }
    }
    fn call(
        &self,
        task: &Task,
        step: &Step,
        d: &Dispatch,
        adapter: &mut impl Adapter,
        joined: Option<Output>,
    ) -> Result<EffectOutcome> {
        if d.invocation != task.id
            || d.prog != task.prog
            || d.neuron != self.profile.subject
            || d.network != self.profile.network
            || d.arguments != handle(task.step)?
            || step.admission != task.admission
        {
            return Err("worker task/step mismatch".into());
        }
        let context_id = if matches!(step.phase, Phase::Model) {
            task.control.context
        } else {
            step.context
        };
        let context = self.context(context_id)?;
        self.context_scope(&context)?;
        let attempt = Attempt {
            operation: d.operation,
            attempt: d.attempt,
            step: task.step,
            context: context_id,
            control: task.control.revision,
            phase: step.phase.clone(),
        };
        let (head, mut state) = self.store.load()?;
        state
            .tasks
            .get_mut(&hex(&task.id))
            .ok_or("missing task")?
            .pending = Some(attempt.clone());
        // Already inside Authority::with_current: do not recursively acquire it.
        self.store.save(head, &state, vec![])?;
        let input: Input = self.store.read(task.input, "soma/input/1")?;
        let request = Request {
            task: task.id,
            operation: d.operation,
            attempt: d.attempt,
            context,
            input: input.text,
            phase: step.phase.clone(),
            history: self.observations(task.id)?,
            steering: task.control.steering.clone(),
            memory: self.memory(&self.context(context_id)?)?,
        };
        let outcome =
            if !matches!(step.phase, Phase::Model) && task.control.revision != step.control {
                AdapterOutcome::Observed(Output::Superseded)
            } else if step.round >= request.context.rounds {
                AdapterOutcome::Observed(Output::Failed {
                    reason: "task round limit".into(),
                })
            } else if let Some(output) = joined {
                AdapterOutcome::Observed(output)
            } else {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| adapter.call(&request)))
                    .unwrap_or(AdapterOutcome::Pending)
            };
        match outcome {
            AdapterOutcome::Pending => Ok(EffectOutcome::Unknown("adapter outcome pending".into())),
            AdapterOutcome::Observed(output) => {
                let observed = self.observe_locked(task.id, attempt, output)?;
                Self::effect(&observed)
            }
        }
    }
    fn effect(observed: &Observed) -> Result<EffectOutcome> {
        match &observed.output {
            Output::Failed { reason } => Ok(EffectOutcome::Failed(reason.clone())),
            _ => Ok(EffectOutcome::Value(match observed.next {
                Some(next) => handle(next)?,
                None => zero(),
            })),
        }
    }
    fn reconcile(&self, id: Id, observed: &Observed) -> Result<()> {
        match Self::effect(observed)? {
            EffectOutcome::Value(value) => self
                .engine
                .record_outcome(
                    self.profile.subject,
                    id,
                    observed.attempt.operation,
                    observed.attempt.attempt,
                    value,
                )
                .map_err(err)?,
            EffectOutcome::Failed(reason) => self
                .engine
                .record_failure(
                    self.profile.subject,
                    id,
                    observed.attempt.operation,
                    observed.attempt.attempt,
                    reason,
                )
                .map_err(err)?,
            EffectOutcome::Unknown(_) => unreachable!(),
        };
        Ok(())
    }
    /// Trusted adapter receipt ingestion, never a tool exposed to the model.
    /// It retains an already dispatched observation after cancellation/revocation.
    /// Engine reconciliation still requires current authority and never replays.
    pub fn observe(&self, id: Id, operation: Id, attempt: Id, output: Output) -> Result<()> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let task = self.task(id)?;
        let raw_output = self.store.content("soma/adapter-output/1", &output)?.id();
        if let Some(prior) = self
            .observations(id)?
            .into_iter()
            .find(|o| o.attempt.operation == operation)
        {
            return if prior.attempt.attempt == attempt && prior.raw_output == raw_output {
                Ok(())
            } else {
                Err("conflicting adapter receipt".into())
            };
        }
        let pending = task.pending.ok_or("no captured adapter request")?;
        if pending.operation != operation || pending.attempt != attempt {
            return Err("adapter receipt lineage mismatch".into());
        }
        self.observe_locked(id, pending, output)?;
        Ok(())
    }
    fn observe_locked(&self, id: Id, attempt: Attempt, output: Output) -> Result<Observed> {
        let (head, mut state) = self.store.load()?;
        let task = state.tasks.get_mut(&hex(&id)).ok_or("missing task")?;
        if task.pending.as_ref() != Some(&attempt) {
            return Err("observation request mismatch".into());
        }
        let context = self.context(attempt.context)?;
        let step: Step = self.store.read(attempt.step, "soma/step/1")?;
        let raw_output = self.store.content("soma/adapter-output/1", &output)?.id();
        let output = match self.validate_output(&attempt.phase, &context, &output) {
            Ok(()) => output,
            Err(_) => Output::Failed {
                reason: "adapter output rejected by admitted schema or limits".into(),
            },
        };
        for prior in &task.observations {
            let prior: Observed = self.store.read(*prior, "soma/observation/1")?;
            if prior.attempt.operation == attempt.operation {
                return if prior.attempt == attempt && prior.raw_output == raw_output {
                    Ok(prior)
                } else {
                    Err("conflicting adapter observation".into())
                };
            }
        }
        if task.observations.len() >= MAX_ROUNDS as usize * 2 + 1 {
            return Err("observation capacity".into());
        }
        let next_phase = match &output {
            Output::Model {
                decision: Decision::Final { answer },
            } => {
                if !task.control.cancelled && task.control.revision != attempt.control {
                    Phase::Model
                } else {
                    task.answer = Some(answer.clone());
                    Phase::Terminal
                }
            }
            Output::Model {
                decision:
                    Decision::Tool {
                        name,
                        schema,
                        arguments,
                    },
            } => Phase::Tool {
                name: name.clone(),
                schema: *schema,
                arguments: arguments.clone(),
            },
            Output::Model {
                decision: Decision::Delegate { children, join },
            } => Phase::Join {
                children: children.clone(),
                policy: join.clone(),
            },
            Output::Failed { reason } => {
                task.failure = Some(reason.clone());
                Phase::Terminal
            }
            _ => Phase::Model,
        };
        let mut content = Vec::new();
        let next = if next_phase == Phase::Terminal {
            None
        } else {
            let context = if output == Output::Superseded {
                task.control.context
            } else {
                attempt.context
            };
            let next = self.store.content(
                "soma/step/1",
                &Step {
                    admission: task.admission,
                    previous: Some(attempt.step),
                    context,
                    round: step.round.checked_add(1).ok_or("round overflow")?,
                    control: attempt.control,
                    phase: next_phase,
                },
            )?;
            let id = next.id();
            content.push(next);
            task.step = id;
            Some(id)
        };
        let observed = Observed {
            attempt,
            output,
            raw_output,
            next,
        };
        let record = self.store.content("soma/observation/1", &observed)?;
        task.observations.push(record.id());
        content.push(record);
        self.store.save(head, &state, content)?;
        Ok(observed)
    }
    fn validate_output(&self, phase: &Phase, context: &Context, output: &Output) -> Result<()> {
        match (phase, output) {
            (_, Output::Failed { reason }) => {
                if reason.len() > 4096 {
                    return Err("adapter failure limit".into());
                }
            }
            (_, Output::Superseded) => {}
            (Phase::Model, Output::Model { decision }) => match decision {
                Decision::Final { answer } => {
                    text(answer)?;
                }
                Decision::Tool {
                    name,
                    schema,
                    arguments,
                } => {
                    let tool = context
                        .tools
                        .iter()
                        .find(|t| t.name == *name && t.schema == *schema)
                        .ok_or("tool not admitted")?;
                    if arguments.len() > tool.max_arguments as usize {
                        return Err("tool argument limit".into());
                    }
                }
                Decision::Delegate { children, .. } => {
                    if children.is_empty() || children.len() > 16 {
                        return Err("delegation limit".into());
                    }
                    for child in children {
                        text(&child.input)?;
                        if child.allowance == 0
                            || child.allowance > 1_000_000
                            || child.disclosure.len() > 64
                            || child
                                .disclosure
                                .iter()
                                .any(|s| !context.disclosure.contains(s))
                        {
                            return Err("child scope/allowance".into());
                        }
                    }
                }
            },
            (Phase::Tool { name, schema, .. }, Output::Tool { value }) => {
                let tool = context
                    .tools
                    .iter()
                    .find(|t| t.name == *name && t.schema == *schema)
                    .ok_or("tool no longer admitted")?;
                if value.len() > tool.max_result as usize {
                    return Err("tool result limit".into());
                }
            }
            (Phase::Join { children, .. }, Output::Joined { children: results }) => {
                if results.len() != children.len() {
                    return Err("join result count".into());
                }
            }
            _ => return Err("output phase mismatch".into()),
        };
        Ok(())
    }
}

/// Read-only application view: no key, activation, worker placement or budget.
pub struct TaskReader {
    store: Store,
    database: Database,
    subject: Id,
    network: Id,
}
impl TaskReader {
    pub fn status(&self, id: Id) -> Result<Status> {
        self.task(id)?;
        let view = Neuron::new(Graph::from_database(self.database.clone()), Rune)
            .inspect(self.subject)
            .map_err(err)?;
        if view.state.network != self.network {
            return Err("task runtime network mismatch".into());
        }
        view.state
            .invocations
            .get(&id)
            .map(|job| job.status)
            .ok_or("missing or archived task runtime".into())
    }
    pub fn input(&self, id: Id) -> Result<Input> {
        self.store.read(self.task(id)?.input, "soma/input/1")
    }
    pub fn new(database: Database, subject: Id, network: Id) -> Result<Self> {
        Ok(Self {
            store: Store::new(database.clone(), subject, network)?,
            database,
            subject,
            network,
        })
    }
    pub fn tasks(&self) -> Result<Vec<Task>> {
        Ok(self.store.load()?.1.tasks.into_values().collect())
    }
    pub fn task(&self, id: Id) -> Result<Task> {
        self.store
            .load()?
            .1
            .tasks
            .remove(&hex(&id))
            .ok_or("missing Soma task".into())
    }
    pub fn context(&self, id: Id) -> Result<Context> {
        self.store.read(id, "soma/context/1")
    }
    pub fn observations(&self, id: Id) -> Result<Vec<Observed>> {
        self.task(id)?
            .observations
            .iter()
            .map(|id| self.store.read(*id, "soma/observation/1"))
            .collect()
    }
    pub fn schedules(&self) -> Result<Vec<Schedule>> {
        Ok(self.store.load()?.1.schedules.into_values().collect())
    }
    pub fn prepared(&self) -> Result<Vec<Input>> {
        self.store
            .load()?
            .1
            .prepared
            .values()
            .filter(|p| p.task.is_none())
            .map(|p| self.store.read(p.input, "soma/input/1"))
            .collect()
    }
}
