# soma-agent

Durable goals/tasks strategy over existing neuron progs, Rune continuations and
explicit local workers. This crate has no signing key, UI dependency or new VM.
The owning contract is [neuron-tasks](../specs/neuron-tasks.md).

The host opens `Agent` with an existing `Authority`, shared Cybergraph Database
and explicit `Profile`. `submit` records a context/input manifest before engine
admission; `poll` runs one bounded VM slice or adapter handoff. A pending effect
returns its operation/attempt; `observe` retains its trusted result and the next
poll reconciles without another physical call. `TaskReader` inspects the same
store without activation, placement or custody. Cyb's common Host constructs
execution from an actual controlled robot attachment.

`control`, schedules/due occurrences, child joins and learning proposals use the
same durable records. `Tools<ModelProvider>` supplies bounded text hashing and a
Unix regular-file read capability. Provider fixtures in tests are explicitly
fixtures. The native cy CLI and Bevy entry points use this library through the
common cyb Host. Full Hermes provider/tool parity remains a separate roadmap. Implementation and release evidence are
tracked in [the convergence audit](../../soft3/audit/neuron-cell/implementation.md).

Run `cargo test --offline` here. Tests exercise the real Rune runtime, worker
claims, signatures and shared BBG storage, including closing/reopening the store,
actual file reads, controls and fault boundaries. No remote service is required.
