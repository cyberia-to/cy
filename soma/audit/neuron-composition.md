# Neuron composition evidence

Local checkout evidence, 2026-09-12. This covers migration package P09 and its
native local G10 scenarios. It is not an upstream Hermes parity audit. The
[accepted task contract](../specs/neuron-tasks.md) and
[provider profile](../specs/local-provider.md) define the boundaries.

| Agent requirement | Owner and executable scenario | Observed evidence |
|---|---|---|
| Durable model/tool work | Soma tasks + neuron/Rune: actual file read, suspension, close/reopen, exact receipt, completion | agent/tests/tasks.rs tool_suspend_restart; soma-agent-final.log |
| Unknown effect and lost receipt | Neuron claims + Soma observation-before-engine publication; panic and fault recovery without another adapter call | observation_before_engine_commit, adapter_panic, admission_and_schedule_receipt_faults |
| Independent work | Multiple prog slots under the same neuron while one operation waits | tool_suspend_restart scenario |
| Context/model/soul capture | Immutable context and control CAS, old pending response retained, next model uses changed revision | context_change_and_steering; steering_during_pending_final; terminal_application_observation |
| Cancel and child joins | Soma poll_tree + neuron reservations, all-required/first-success/best-effort, parent cancel before child dispatch | all_join_policies; common_tree_driver; required_join_failure |
| Child admission crash | Original engine nonce lookup, missing application receipt, then parent cancellation and restart | child_receipt_missing_from_application; soma-child-receipt-tests.log |
| Time/schedule recovery | Stable occurrence nonce across preparation, admission and cursor faults; explicit disabling | schedule_restart; admission_and_schedule_receipt_faults |
| Tools and disclosure | Exact schema, bounded output, real no-follow file open, subset memory/tool scopes, malformed result retention | tool_schema_scope_output_bounds; memory/learning scenario in tasks.rs |
| Memory and learning | Pinned source scope/head/content, proposal-only learning with terminal task provenance | memory/learning scenario; no implicit soul or grant update |
| Authority and placement | Actual cyb Host/Registry/native custody, selection changes, revoked independent adapter and new worker generation | cyb/core/tests/runtime_authority.rs and soma_host.rs |
| Actual local inference | glia weight checksum, real generation, correlated stream/result and durable engine completion | kernel/tests/durable_provider.rs; soma-real-provider-tests.log |
| CLI and Bevy bodies | Actual executables, isolated HOME and key, real weights, durable result after stopping body, exact terminal run without new inference | cyb/harness/tasks.py; soma-real-bodies.log; screenshot inspected |
| GUI authorship | Out-of-order two-subject results, selection/model capture, rejected mismatched admission, child/tool stream filtering | three soma_bridge tests; soma-bevy-correlation-tests.log |
| Headless controls and read views | Real cy task processes, context/steer CAS, cancellation, revoked explicit-attachment history, no new key | cyb/cli/tests/tasks.rs; p09-composition-tests.log |
| Private state | Vault-owned encrypted notes, subject/network scopes, exact local payment, change and crash recovery, no replay | cyb/core private_notes tests and cli/tests/notes.rs |

The final task-library run contains 15 integration tests plus the real Rune seam.
The kernel unit run contains seven tests, including the existing real-model,
grounding and model-switch scenarios. The combined cyb/core/CLI suites also cover
the preceding registry, public history, private vault, relay and mirror boundaries.
Exact counts and subsequent reruns belong to the
[shared implementation ledger](../../soft3/audit/neuron-cell/implementation.md).
Logs referenced above are in that ledger's implementation-baseline directory.

The actual CLI and GUI body fixture used the existing local
qwen3-0.6b-abl.model (430,237,651 bytes). Both reopened one completed task with its
original attachment, observations and answer; running its terminal receipt did
not create another observation or identity. The Bevy screenshot displayed the
captured question and completed answer. All fixture process groups were stopped.

Explicit profile limits remain: trusted local host adapters, cooperative GPU
deadlines, one native execution network per subject/database, bounded retained
catalogs, no automatic unknown-effect retry and no remote writer consensus.
Convenience ask pins a workspace root label; it does not claim a filesystem
snapshot. Public GUI exchange links are a later authorized projection; the
completed task record remains available if that projection is interrupted.
Broader providers, write tools, channels, autonomous markets and proof-native
inference remain staged product requirements in the separate agent work.
