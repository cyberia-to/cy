use serde::{Deserialize, Serialize};
pub type Id = [u8; 32];
pub type Result<T> = std::result::Result<T, String>;
pub const HOST: u64 = 0xAC75_0000_0000_0006;
pub const MAX_RECORD: usize = 8 * 1024 * 1024;
pub const MAX_TEXT: usize = 64 * 1024;
pub const MAX_TASKS: usize = 512;
pub const MAX_ROUNDS: u32 = 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub provider: String,
    pub name: String,
    pub revision: Id,
    pub max_tokens: u32,
    pub parameters: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub name: String,
    pub schema: Id,
    pub max_arguments: u32,
    pub max_result: u32,
    pub effect: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub scope: String,
    pub head: Id,
    pub content: Id,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub subject: Id,
    pub network: Id,
    pub attachment: Id,
    pub binding_revision: u64,
    pub policy: Id,
    pub epoch: u64,
    pub now: u64,
    pub soul: Id,
    pub model: Model,
    pub tools: Vec<Tool>,
    pub workspace: String,
    pub workspace_revision: Id,
    pub memory: Vec<Memory>,
    pub disclosure: Vec<String>,
    pub conversation: Option<Id>,
    pub goal: Option<Id>,
    pub plan: Option<Id>,
    pub rounds: u32,
}
impl Context {
    pub fn validate(&self) -> Result<()> {
        if self.rounds == 0
            || self.rounds > MAX_ROUNDS
            || self.model.max_tokens == 0
            || self.model.max_tokens > 32768
            || self.tools.len() > 32
            || self.memory.len() > 64
            || self.disclosure.len() > 64
            || !std::path::Path::new(&self.workspace).is_absolute()
            || self.workspace.len() > 4096
            || self.model.parameters.len() > 4096
        {
            return Err("context limits".into());
        }
        label(&self.model.provider)?;
        label(&self.model.name)?;
        let mut names = std::collections::BTreeSet::new();
        for tool in &self.tools {
            label(&tool.name)?;
            label(&tool.effect)?;
            if !names.insert(&tool.name)
                || tool.max_arguments == 0
                || tool.max_arguments as usize > MAX_TEXT
                || tool.max_result == 0
                || tool.max_result as usize > MAX_TEXT
            {
                return Err("tool limits".into());
            }
        }
        for scope in &self.disclosure {
            label(scope)?;
        }
        for memory in &self.memory {
            label(&memory.scope)?;
            if !self.disclosure.contains(&memory.scope) {
                return Err("memory disclosure not admitted".into());
            }
        }
        Ok(())
    }
}
pub(crate) fn label(s: &str) -> Result<()> {
    if s.is_empty() || s.len() > 256 || s.chars().any(char::is_control) {
        Err("invalid label".into())
    } else {
        Ok(())
    }
}
pub(crate) fn text(s: &str) -> Result<()> {
    if s.len() > MAX_TEXT {
        Err("text limit".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub nonce: Id,
    pub text: String,
    pub context: Context,
    pub allowance: u64,
    pub parent: Option<Id>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Join {
    AllRequired,
    FirstSuccess,
    BestEffort,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Child {
    pub input: String,
    pub allowance: u64,
    /// A subset of the parent's explicitly admitted source scopes.
    pub disclosure: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    Final {
        answer: String,
    },
    Tool {
        name: String,
        schema: Id,
        arguments: String,
    },
    Delegate {
        children: Vec<Child>,
        join: Join,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase {
    Model,
    Tool {
        name: String,
        schema: Id,
        arguments: String,
    },
    Join {
        children: Vec<Child>,
        policy: Join,
    },
    Terminal,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Output {
    Model { decision: Decision },
    Tool { value: String },
    Joined { children: Vec<ChildResult> },
    Superseded,
    Failed { reason: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildResult {
    pub task: Id,
    pub status: String,
    pub answer: Option<String>,
}
/// Pending is not failure. A callback cannot infer that an effect did not occur
/// from a timeout. Its matching observation is supplied later by the host.
pub enum AdapterOutcome {
    Observed(Output),
    Pending,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub task: Id,
    pub operation: Id,
    pub attempt: Id,
    pub context: Context,
    pub input: String,
    pub phase: Phase,
    pub history: Vec<Observed>,
    pub steering: Vec<String>,
    pub memory: Vec<(Memory, String)>,
}
pub trait Adapter {
    /// Called only inside a fresh neuron worker dispatch. Implementations must
    /// enforce their declared wall time, output and device limits.
    fn call(&mut self, request: &Request) -> AdapterOutcome;
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub admission: Id,
    pub previous: Option<Id>,
    pub context: Id,
    pub round: u32,
    pub control: u64,
    pub phase: Phase,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub operation: Id,
    pub attempt: Id,
    pub step: Id,
    pub context: Id,
    pub control: u64,
    pub phase: Phase,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observed {
    pub attempt: Attempt,
    pub output: Output,
    pub raw_output: Id,
    pub next: Option<Id>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Control {
    pub revision: u64,
    pub context: Id,
    pub steering: Vec<String>,
    pub cancelled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub prog: Id,
    pub admission: Id,
    pub input: Id,
    pub step: Id,
    pub control: Control,
    pub pending: Option<Attempt>,
    pub observations: Vec<Id>,
    pub children: Vec<Id>,
    pub answer: Option<String>,
    pub failure: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Running,
    Waiting,
    Unknown { operation: Id, attempt: Id },
    Completed(String),
    Cancelled,
    Failed(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Id,
    pub due: u64,
    pub interval: Option<u64>,
    pub remaining: u32,
    pub occurrence: u32,
    pub enabled: bool,
    pub input: Input,
    pub receipts: Vec<Id>,
}
