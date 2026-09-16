use crate::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug)]
pub enum Change {
    Steer(String),
    Context(Context),
    Cancel,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningProposal {
    pub task: Id,
    pub context: Id,
    pub observations: Vec<Id>,
    pub text: String,
    pub checks: Vec<String>,
}
impl<A: Authority> Agent<A> {
    pub fn control(&self, id: Id, expected: u64, change: Change) -> Result<Control> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let (head, mut state) = self.store.load()?;
        let task = state.tasks.get_mut(&hex(&id)).ok_or("missing task")?;
        if task.control.revision != expected {
            return Err("stale control revision".into());
        }
        if (task.answer.is_some() || task.failure.is_some()) && !matches!(change, Change::Cancel) {
            return Err("terminal result already observed; submit follow-up work".into());
        }
        let view = self.engine.inspect(self.profile.subject).map_err(err)?;
        if view
            .state
            .invocations
            .get(&id)
            .ok_or("missing invocation")?
            .status
            .terminal()
        {
            return Err("task already terminal".into());
        }
        let mut content = vec![];
        match change {
            Change::Steer(message) => {
                text(&message)?;
                if task.control.cancelled
                    || task.control.steering.len() >= 16
                    || task.control.steering.iter().map(String::len).sum::<usize>() + message.len()
                        > MAX_TEXT
                {
                    return Err("steering closed/limit".into());
                }
                task.control.steering.push(message);
            }
            Change::Context(context) => {
                if task.control.cancelled {
                    return Err("cancelled task context".into());
                }
                self.context_scope(&context)?;
                self.memory(&context)?;
                let record = self.store.content("soma/context/1", &context)?;
                task.control.context = record.id();
                content.push(record);
            }
            Change::Cancel => task.control.cancelled = true,
        }
        task.control.revision = expected
            .checked_add(1)
            .ok_or("control revision exhausted")?;
        let control = task.control.clone();
        let prog = task.prog;
        let record = self.store.content("soma/control/1", &(id, &control))?;
        content.push(record);
        self.publish(head, &state, content, "soma/control", Some(prog), Some(id))?;
        Ok(control)
    }
    pub(crate) fn mark_cancelled(&self, id: Id) -> Result<()> {
        let (head, mut state) = self.store.load()?;
        let task = state.tasks.get_mut(&hex(&id)).ok_or("missing child")?;
        if task.control.cancelled {
            return Ok(());
        }
        task.control.cancelled = true;
        task.control.revision = task
            .control
            .revision
            .checked_add(1)
            .ok_or("control revision exhausted")?;
        let prog = task.prog;
        self.publish(
            head,
            &state,
            vec![],
            "soma/join-cancel",
            Some(prog),
            Some(id),
        )?;
        Ok(())
    }
    pub(crate) fn cancel_children(&self, task: &Task) -> Result<bool> {
        let mut ready = true;
        for id in &task.children {
            let child = self.task(*id)?;
            let view = self.engine.inspect(self.profile.subject).map_err(err)?;
            let job = view
                .state
                .invocations
                .get(id)
                .ok_or("missing child invocation")?;
            if job.status.terminal() {
                continue;
            }
            self.mark_cancelled(*id)?;
            if !self.cancel_children(&child)? {
                ready = false;
                continue;
            }
            if let Some(p) = &job.pending {
                if p.stage == 1 {
                    if let Some(observed) = self.observations(*id)?.into_iter().find(|o| {
                        o.attempt.operation == p.id && Some(o.attempt.attempt) == p.attempt
                    }) {
                        self.reconcile(*id, &observed)?;
                    } else {
                        ready = false;
                        continue;
                    }
                }
            }
            self.engine
                .cancel(
                    self.profile.subject,
                    *id,
                    "parent control/join cancellation".into(),
                )
                .map_err(err)?;
        }
        Ok(ready)
    }
    pub(crate) fn join_locked(&self, task: &Task, step: &Step) -> Result<Option<Output>> {
        let Phase::Join { children, policy } = &step.phase else {
            return Err("not a join step".into());
        };
        // Changing context invalidates adoption and stops unused child work.
        if task.control.revision != step.control {
            for child in &task.children {
                self.mark_cancelled(*child)?;
            }
            return if self.cancel_children(task)? {
                Ok(Some(Output::Superseded))
            } else {
                Ok(None)
            };
        }
        let context = self.context(step.context)?;
        let view = self.engine.inspect(self.profile.subject).map_err(err)?;
        let parent = view
            .state
            .invocations
            .get(&task.id)
            .ok_or("missing parent")?;
        let total = children.iter().try_fold(0u64, |n, c| {
            n.checked_add(c.allowance).ok_or("child allowance overflow")
        })?;
        if children.len() >= self.profile.slots || total >= parent.limit {
            return Ok(Some(Output::Failed {
                reason: "delegation exceeds program pool or parent allowance".into(),
            }));
        }
        let (_, catalog) = self.store.load()?;
        let remaining = children
            .iter()
            .enumerate()
            .try_fold(0u64, |n, (index, child)| {
                let nonce = derive(
                    "soma/child/1",
                    &[&task.id, &task.step, &(index as u64).to_le_bytes()],
                );
                let admitted = catalog
                    .prepared
                    .get(&hex(&nonce))
                    .and_then(|p| p.task)
                    .is_some();
                n.checked_add(if admitted { 0 } else { child.allowance })
                    .ok_or("child allowance overflow")
            })?;
        if parent
            .available()
            .map_err(err)?
            .saturating_sub(parent.reserved)
            < remaining.saturating_add(2000)
        {
            if !self.cancel_children(task)? {
                return Ok(None);
            }
            return Ok(Some(Output::Failed {
                reason: "insufficient parent budget for children and adoption".into(),
            }));
        }
        let mut ids = vec![];
        for (index, child) in children.iter().enumerate() {
            let nonce = derive(
                "soma/child/1",
                &[&task.id, &task.step, &(index as u64).to_le_bytes()],
            );
            let mut context = context.clone();
            context.disclosure = child.disclosure.clone();
            context
                .memory
                .retain(|m| context.disclosure.contains(&m.scope));
            let input = Input {
                nonce,
                text: child.input.clone(),
                context,
                allowance: child.allowance,
                parent: Some(task.id),
            };
            let id = match self.submit_locked(input) {
                Ok(id) => id,
                Err(e) if e == "task pool occupied" => return Ok(None),
                Err(e) => return Err(e),
            };
            ids.push(id);
            let (head, mut state) = self.store.load()?;
            let parent = state
                .tasks
                .get_mut(&hex(&task.id))
                .ok_or("missing parent")?;
            if !parent.children.contains(&id) {
                parent.children.push(id);
                self.publish(
                    head,
                    &state,
                    vec![],
                    "soma/delegated",
                    Some(task.prog),
                    Some(task.id),
                )?;
            }
        }
        let mut results = Vec::new();
        let view = self.engine.inspect(self.profile.subject).map_err(err)?;
        let mut success = false;
        let mut failed = false;
        let mut waiting = false;
        for id in &ids {
            let child = self.task(*id)?;
            let job = view
                .state
                .invocations
                .get(id)
                .ok_or("missing child invocation")?;
            success |= job.status == Status::Completed && child.answer.is_some();
            failed |= job.status.terminal() && job.status != Status::Completed;
            waiting |= !job.status.terminal();
            results.push(ChildResult {
                task: *id,
                status: job.status.name().into(),
                answer: child.answer,
            });
        }
        if waiting
            && (policy == &Join::FirstSuccess && success || policy == &Join::AllRequired && failed)
        {
            for id in &ids {
                if !view.state.invocations[id].status.terminal() {
                    self.mark_cancelled(*id)?;
                }
            }
            self.cancel_children(&self.task(task.id)?)?;
            return Ok(None);
        }
        if waiting {
            return Ok(None);
        }
        if policy == &Join::AllRequired && failed || policy == &Join::FirstSuccess && !success {
            Ok(Some(Output::Failed {
                reason: "required child result unavailable".into(),
            }))
        } else {
            Ok(Some(Output::Joined { children: results }))
        }
    }
    pub fn schedule(&self, schedule: Schedule) -> Result<Id> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        self.context_scope(&schedule.input.context)?;
        text(&schedule.input.text)?;
        if schedule.remaining == 0
            || schedule.remaining > 256
            || schedule.occurrence != 0
            || !schedule.receipts.is_empty()
            || schedule.interval == Some(0)
            || schedule.interval.is_none() && schedule.remaining != 1
            || schedule.input.parent.is_some()
        {
            return Err("schedule bounds".into());
        }
        let (head, mut state) = self.store.load()?;
        if let Some(existing) = state.schedules.get(&hex(&schedule.id)) {
            return if existing == &schedule {
                Ok(schedule.id)
            } else {
                Err("schedule id conflict".into())
            };
        }
        let id = schedule.id;
        state.schedules.insert(hex(&id), schedule);
        self.publish(head, &state, vec![], "soma/schedule", None, None)?;
        Ok(id)
    }
    pub fn schedules(&self) -> Result<Vec<Schedule>> {
        Ok(self.store.load()?.1.schedules.into_values().collect())
    }
    pub fn enable_schedule(&self, id: Id, expected_occurrence: u32, enabled: bool) -> Result<()> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let (head, mut state) = self.store.load()?;
        let schedule = state
            .schedules
            .get_mut(&hex(&id))
            .ok_or("missing schedule")?;
        if schedule.occurrence != expected_occurrence {
            return Err("stale schedule occurrence".into());
        }
        schedule.enabled = enabled;
        self.publish(head, &state, vec![], "soma/schedule-state", None, None)?;
        Ok(())
    }
    /// Admit at most one due occurrence. Exact deterministic retries survive
    /// a crash across the engine receipt and the scheduler cursor write.
    pub fn due(&self, now: u64) -> Result<Option<Id>> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        let (_, state) = self.store.load()?;
        let Some(schedule) = state
            .schedules
            .values()
            .filter(|s| s.enabled && s.remaining > 0 && s.due <= now)
            .min_by_key(|s| (s.due, s.id))
            .cloned()
        else {
            return Ok(None);
        };
        let mut input = schedule.input.clone();
        input.nonce = derive(
            "soma/occurrence/1",
            &[&schedule.id, &schedule.occurrence.to_le_bytes()],
        );
        input.context.now = schedule.due;
        let task = self.submit_locked(input)?;
        let (head, mut state) = self.store.load()?;
        let current = state
            .schedules
            .get_mut(&hex(&schedule.id))
            .ok_or("missing schedule")?;
        if current != &schedule {
            return Err("schedule changed during admission".into());
        }
        current.receipts.push(task);
        current.occurrence = current
            .occurrence
            .checked_add(1)
            .ok_or("occurrence exhausted")?;
        current.remaining -= 1;
        if current.remaining > 0 {
            current.due = current
                .due
                .checked_add(current.interval.ok_or("missing interval")?)
                .ok_or("schedule time exhausted")?;
        } else {
            current.enabled = false;
        }
        self.publish(head, &state, vec![], "soma/occurrence", None, Some(task))?;
        Ok(Some(task))
    }
    /// Records a proposal only. Soul/skill/grant adoption is a separate host
    /// version change; calling this method never edits any of those owners.
    pub fn propose_learning(&self, task: Id, proposal: String, checks: Vec<String>) -> Result<Id> {
        let _guard = self
            .store
            .lock
            .lock()
            .map_err(|_| "Soma coordinator poisoned")?;
        text(&proposal)?;
        if checks.is_empty() || checks.len() > 32 {
            return Err("learning checks required/bounded".into());
        }
        for check in &checks {
            label(check)?;
        }
        let (head, mut state) = self.store.load()?;
        let task = state.tasks.get(&hex(&task)).ok_or("missing task")?;
        let view = self.engine.inspect(self.profile.subject).map_err(err)?;
        if !view
            .state
            .invocations
            .get(&task.id)
            .ok_or("missing invocation")?
            .status
            .terminal()
        {
            return Err("learning needs terminal observation".into());
        }
        let originating_context = match task.observations.last() {
            Some(id) => {
                self.store
                    .read::<Observed>(*id, "soma/observation/1")?
                    .attempt
                    .context
            }
            None => task.control.context,
        };
        let proposal = LearningProposal {
            task: task.id,
            context: originating_context,
            observations: task.observations.clone(),
            text: proposal,
            checks,
        };
        let content = self.store.content("soma/learning-proposal/1", &proposal)?;
        let id = content.id();
        let prog = task.prog;
        if !state.proposals.contains(&id) {
            state.proposals.push(id);
        }
        self.publish(
            head,
            &state,
            vec![content],
            "soma/learning-proposal",
            Some(prog),
            Some(proposal.task),
        )?;
        Ok(id)
    }
    pub fn learning(&self) -> Result<Vec<LearningProposal>> {
        self.store
            .load()?
            .1
            .proposals
            .iter()
            .map(|id| self.store.read(*id, "soma/learning-proposal/1"))
            .collect()
    }
}
