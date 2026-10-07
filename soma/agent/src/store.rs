use crate::types::*;
use cybergraph::{
    application::{ApplicationGraph, Database, Head, Proposal},
    content::{Codec, Content},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
pub fn hash(bytes: &[u8]) -> Id {
    *hemera::hash(bytes).as_bytes()
}
pub fn hex(id: &Id) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn derive(domain: &str, parts: &[&[u8]]) -> Id {
    let mut h = hemera::Hasher::new();
    h.update(domain.as_bytes());
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    *h.finalize().as_bytes()
}
pub(crate) fn err(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Prepared {
    pub input: Id,
    pub prog: Id,
    pub step: Id,
    pub task: Option<Id>,
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct State {
    pub slots: Vec<Id>,
    pub prepared: BTreeMap<String, Prepared>,
    pub tasks: BTreeMap<String, Task>,
    pub schedules: BTreeMap<String, Schedule>,
    pub proposals: Vec<Id>,
}
#[derive(Serialize, Deserialize)]
struct Record<T> {
    schema: String,
    value: T,
}
pub(crate) struct Store {
    pub graph: ApplicationGraph,
    pub lock: Arc<Mutex<()>>,
    namespace: Id,
}
impl Store {
    pub fn new(database: Database, subject: Id, network: Id) -> Result<Self> {
        let namespace = derive("soma/tasks/1", &[&subject, &network]);
        let lock = database.coordination_lock(namespace).map_err(err)?;
        Ok(Self {
            graph: ApplicationGraph::from_database(database),
            lock,
            namespace,
        })
    }
    pub fn content<T: Serialize>(&self, schema: &str, value: &T) -> Result<Content> {
        // A capped writer prevents accidental unlimited serialization allocation.
        struct Capped(Vec<u8>);
        impl std::io::Write for Capped {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                if self.0.len() + b.len() > MAX_RECORD {
                    return Err(std::io::Error::other("Soma record limit"));
                }
                self.0.extend_from_slice(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut bytes = Capped(Vec::new());
        serde_json::to_writer(
            &mut bytes,
            &Record {
                schema: schema.into(),
                value,
            },
        )
        .map_err(err)?;
        Content::new(Codec::Blob, bytes.0).map_err(err)
    }
    pub fn read<T: DeserializeOwned>(&self, id: Id, schema: &str) -> Result<T> {
        let value = self
            .graph
            .get(&id)
            .map_err(err)?
            .ok_or("missing Soma artifact")?;
        if value.codec() != Codec::Blob || value.bytes().len() > MAX_RECORD {
            return Err("Soma artifact codec/limit".into());
        }
        let record: Record<T> = serde_json::from_slice(value.bytes()).map_err(err)?;
        if record.schema != schema {
            return Err("Soma schema mismatch".into());
        }
        Ok(record.value)
    }
    pub fn load(&self) -> Result<(Option<Head>, State)> {
        let head = self.graph.head(&self.namespace).map_err(err)?;
        let state: State = match head {
            None => State::default(),
            Some(h) => self.read(h.commit, "soma/state/1")?,
        };
        state.validate()?;
        Ok((head, state))
    }
    pub fn save(
        &self,
        expected: Option<Head>,
        state: &State,
        mut content: Vec<Content>,
    ) -> Result<Head> {
        state.validate()?;
        let record = self.content("soma/state/1", state)?;
        let head = Head {
            index: expected.map_or(Ok(0), |h| {
                h.index.checked_add(1).ok_or("Soma revision exhausted")
            })?,
            commit: record.id(),
        };
        content.push(record);
        let content: Vec<_> = content
            .into_iter()
            .map(|c| (c.id(), c))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect();
        self.graph
            .commit(
                &Proposal {
                    namespace: self.namespace,
                    request: derive(
                        "soma/state-write/1",
                        &[&head.commit, &head.index.to_le_bytes()],
                    ),
                    expected,
                    head,
                    content,
                    required: expected.into_iter().map(|h| h.commit).collect(),
                    claims: vec![],
                },
                |_| Ok(()),
            )
            .map_err(err)
    }
}

impl State {
    fn validate(&self) -> Result<()> {
        if self.tasks.len() > MAX_TASKS
            || self.prepared.len() > MAX_TASKS
            || self.slots.len() > 64
            || self.schedules.len() > 64
            || self.proposals.len() > 512
        {
            return Err("Soma catalog capacity".into());
        }
        let slots: std::collections::BTreeSet<_> = self.slots.iter().collect();
        if slots.len() != self.slots.len() {
            return Err("duplicate Soma slot".into());
        }
        for (key, task) in &self.tasks {
            if key != &hex(&task.id)
                || !slots.contains(&task.prog)
                || task.observations.len() > MAX_ROUNDS as usize * 2 + 1
                || task.children.len() > 256
                || task.control.steering.len() > 16
            {
                return Err("invalid Soma task catalog".into());
            }
            if let Some(answer) = &task.answer {
                text(answer)?;
            }
            if let Some(failure) = &task.failure {
                if failure.len() > 4096 {
                    return Err("task fault limit".into());
                }
            }
        }
        Ok(())
    }
}
