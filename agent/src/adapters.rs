//! Explicit adapter composition. Providers have no keys and cannot mint tools.
use crate::*;
pub trait ModelProvider {
    fn generate(&mut self, request: &Request) -> AdapterOutcome;
}
pub struct Tools<M> {
    pub model: M,
}
impl<M: ModelProvider> Adapter for Tools<M> {
    fn call(&mut self, request: &Request) -> AdapterOutcome {
        match &request.phase {
            Phase::Model => self.model.generate(request),
            Phase::Tool {
                name,
                schema,
                arguments,
            } => {
                if name == "text.hash" && *schema == hash(b"soma/tool/text.hash/1") {
                    AdapterOutcome::Observed(Output::Tool {
                        value: hex(&hash(arguments.as_bytes())),
                    })
                } else if name == "workspace.read" && *schema == hash(b"soma/tool/workspace.read/1")
                {
                    match read_workspace(request, arguments) {
                        Ok(value) => AdapterOutcome::Observed(Output::Tool { value }),
                        Err(_) => AdapterOutcome::Observed(Output::Failed {
                            reason: "workspace read denied, changed or exceeds limits".into(),
                        }),
                    }
                } else {
                    AdapterOutcome::Observed(Output::Failed {
                        reason: "tool adapter unavailable for admitted schema".into(),
                    })
                }
            }
            _ => AdapterOutcome::Observed(Output::Failed {
                reason: "not an adapter phase".into(),
            }),
        }
    }
}
pub fn hash_tool() -> Tool {
    Tool {
        name: "text.hash".into(),
        schema: hash(b"soma/tool/text.hash/1"),
        max_arguments: MAX_TEXT as u32,
        max_result: 64,
        effect: "pure/1".into(),
    }
}

pub fn read_tool() -> Tool {
    Tool {
        name: "workspace.read".into(),
        schema: hash(b"soma/tool/workspace.read/1"),
        max_arguments: 1024,
        max_result: MAX_TEXT as u32,
        effect: "read/1".into(),
    }
}
fn read_workspace(request: &Request, arguments: &str) -> Result<String> {
    use std::io::Read;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ReadArgs {
        name: String,
        expected: Option<String>,
    }
    let args: ReadArgs = serde_json::from_str(arguments).map_err(|_| "invalid read arguments")?;
    // This initial capability is one regular file in an explicitly admitted
    // directory. Nested paths and symlinks are not part of this tool profile.
    if args.name.is_empty()
        || args.name == "."
        || args.name == ".."
        || args.name.contains(['/', '\\', '\0'])
    {
        return Err("read path outside capability".into());
    }
    #[cfg(unix)]
    {
        use rustix::fs::{Mode, OFlags, open, openat};
        let root = open(
            request.context.workspace.as_str(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| "workspace unavailable")?;
        let fd = openat(
            &root,
            args.name.as_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| "file unavailable")?;
        let file = std::fs::File::from(fd);
        if !file.metadata().map_err(|_| "file metadata")?.is_file() {
            return Err("not a regular file".into());
        }
        let limit = request
            .context
            .tools
            .iter()
            .find(|t| t.name == "workspace.read")
            .ok_or("read capability absent")?
            .max_result as usize;
        let mut bytes = Vec::new();
        file.take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "read failed")?;
        if bytes.len() > limit {
            return Err("file exceeds admitted result limit".into());
        }
        if args
            .expected
            .is_some_and(|expected| expected != hex(&hash(&bytes)))
        {
            return Err("file revision changed".into());
        }
        String::from_utf8(bytes).map_err(|_| "file is not text".into())
    }
    #[cfg(not(unix))]
    {
        let _ = request;
        Err("workspace.read requires the declared Unix fd capability profile".into())
    }
}
