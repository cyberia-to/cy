# Durable tasks over neuron programs

Status: accepted composition contract for P09. This does not claim provider/tool
feature parity with Hermes. The [agent profile](../../neuron/specs/agent.md)
retains the wider inventory; supported adapters and executable evidence must be
listed separately.

## Ownership and execution

Soma owns goals, task inputs, context selection, model/tool strategy, plans,
schedules and learning proposals. Neuron owns subject/prog/invocation state,
continuations, resource accounting, operation claims, cancellation and recovery.
Cybergraph owns the records in the shared BBG database. Robot attachments supply
explicit current authority; Body supplies worker/device placement. Neither a
conversation, task, goal, model attempt nor child has a signing key.

The task executor uses the existing `neuron-rune::Rune` adapter and its bounded
machine checkpoints. It does not introduce another VM. The pinned Rune program
loops over `host(task_step_reference)` until the host returns zero. Each nonzero
reply names an immutable next step; large prompts and results remain graph
artifacts rather than expanded VM literals. Task program/source/ABI hashes and
worker `native-rust/1` / `neuron/host-act/1` descriptors are explicit. The local
worker profile makes no proof, remote consensus or hard preemption claim.

A small explicit pool of progs can run concurrent tasks under one neuron.
Independent mutable prog state must not silently rebase a competing invocation.
An occupied slot queues/refuses admission according to declared capacity. Reusing
an idle slot retains its current revision; task/goal history survives reuse.
Creating slots, child tasks or moving a worker never creates another neuron.

## Records and admission

A versioned context manifest pins subject/network, attachment policy and revision,
now, soul configuration, model/provider/parameters, exact tool schema revisions,
source scopes/observed heads, workspace root/revision, input artifacts, disclosure,
conversation, goal/plan revision and resource bounds. Foreign/watch-only profiles
can be inspected but cannot execute through the native adapter. User input,
retrieved context and tool output remain distinguishable by provenance; context
text cannot change grants or tool definitions.

The durable admission nonce and immutable input manifest precede engine submit.
An exact retry resolves the same invocation; a changed payload conflicts. The
engine invocation identifies the task. Subsequent task steps bind that invocation,
prog, subject/network, prior step, originating operation/attempt, request context,
output artifacts, model/tool phase and remaining round bounds. A worker compares
these fields with its own Dispatch, not with claims inside model output.

Soma records strategy/output observations before returning an engine host result.
The record binds the exact operation/attempt and supports reconciliation if the
process dies between recording the observation and engine result publication.
A task is complete only when both its terminal application result and the engine
terminal state agree. A lost provider/tool outcome stays unknown and blocks a new
physical call; an old effect receipt never becomes another dispatch token.

## Model, tools and controls

Model requests pin provider/model/context and output limits. A model result is
validated structured data: final answer, allowed tool call, or declared delegation.
Malformed/unsupported output fails as an observed adapter result. Tools have exact
names/schema versions, bounded arguments/results, workspace/disclosure contracts
and declared effect/idempotency semantics. A model cannot mint a tool, credential,
network switch or authorization. Real adapters run only at the fresh worker claim
boundary; deterministic test providers are labelled fixtures.

User steering, context/model change and cancellation are durable control records
with monotonically increasing revisions. They take effect at safe boundaries.
Already accepted work retains its original context and provider; its result is
retained with that provenance. A later model request uses the admitted new
revision. Cancellation stops fresh dispatch, retains outstanding attempts and
records their eventual outcome without adopting it as new parent state.

Delegation persists parent-child lineage, grant subset, per-child budget transfer,
input disclosure and join policy. Children use explicit progs of the same subject
unless a different signing authority was requested. All-required, first-success
and best-effort joins name their adoption boundary; early completion cancels or
explicitly detaches remaining work. Joining waits without blocking unrelated
invocations. Parent adoption checks current control/context/workspace revision.

Schedules retain due time, occurrence nonce, input/context, owner and admission
receipt. A crash across due→submit→receipt must produce one invocation for that
occurrence. Recurrence is bounded and advances only with its prior receipt;
disabling a schedule stops new occurrences without erasing existing work.

Learning produces a proposal artifact with originating task/context/outcome and
checks. It never edits soul, skill policy or grants automatically. Adoption is a
separate authorized version change. Memory reads name their source scope and
revision; no process-local conversation vector is the authoritative task record.

## Conformance required before release

| Requirement | Owning boundary | Required scenario |
|---|---|---|
| Resumable model/tool loop | Soma strategy + Rune/neuron | real tool effect, suspend, close/reopen store, result, final answer |
| Unknown effect | Worker + operation journal | loss after dispatch; no replay; exact receipt reconciliation |
| Context/model capture | Soma controls + attachment authority | switch UI/provider while waiting; retain old result; next call uses admitted revision |
| Grant/network/device | Cyb binding adapter + worker | revoke, stale revision/worker, wrong network all deny before physical call |
| Independent work | Neuron prog slots | waiting tool and another runnable task under one subject |
| Delegation | Soma joins + neuron reservations | child restart/result, all join policies, bounded parent adoption |
| Cancel/steer | Soma control record + neuron | queued/running/waiting task, no new effect after cancellation |
| Schedule recovery | Soma/Time | crash before/after submit and cursor persistence, one occurrence |
| Memory/learning | Cybergraph + Soma | scoped retrieval, retained provenance, proposal only until adoption |
| Private state | Vault + wallet owner | secret-free task records and restart-safe private notes |
| Body-independent adapter | Common composition API | the same task contract through headless and Bevy entry points |

Every scenario needs owner test evidence. Rendering a chat answer or passing the
Rune loop seam alone is insufficient for this profile or gate G10.

## Initial native library profile

`soma/agent` is a library inside Soma. Its host supplies an existing Authority,
Database and explicit Profile. Opening execution places a new worker generation;
`TaskReader` is the separate read-only API and never places a worker or activates
a subject. The cyb common Host constructs Profile from the captured attachment
and configured device; neither a model response nor a navigation route can do so.
Existing runtime budget/charges survive reopening regardless of a larger supplied
initial budget. The native engine currently binds one execution network to a
subject in a database; a mismatched attachment fails explicitly.

The profile caps the prog pool at 64 slots, catalog at 512 task admissions,
schedules at 64 with 256 occurrences each, child batches at 16, rounds at 64,
individual text at 64 KiB and a serialized catalog/artifact at 8 MiB. The byte cap
may be reached before the row cap; capacity errors retain prior state. Historical
application records are retained. Automatic pruning/compaction is not implied.
A parent keeps at least 2,000 remaining Rune steps for join adoption before
transferring a new child batch. Instruction charges and provider token/output
limits have separate units; neither represents a token payment or GPU proof.

The task record contains immutable input/step references, a control revision and
per-attempt observations. Adapter output hashes distinguish conflicting replies,
including rejected schema/limit responses. A duplicate old observation can resolve
after a later step completes. A process lost before capturing a physical adapter
request exposes the engine's unknown attempt; only trustworthy reconciliation of
that original operation may resolve it. No automatic call is made from a receipt.
Prepared admissions can be inspected and explicitly recovered using their exact
original manifest. Already recorded admission receipts remain read-only retries.
Direct child submission records lineage in both neuron and Soma. If a crash
separates engine admission from the Soma receipt, recovery looks up that exact
nonce before considering another admission. A subsequent parent cancellation
cannot erase this child; the recovered child inherits cancellation. The common
`poll_tree` observes parent controls before advancing a child and fairly rotates
the bounded tree. This scheduling cursor is a local hint, not a new actor.

A steering/control update while a final model response is outstanding retains
that response but schedules another model boundary before completing. Pending
calls keep their old context. A not-yet-dispatched tool/delegation generated under
an older control revision is superseded; existing child effects are cancelled or
wait for their known outcomes before adoption. First-success joins cancel unused
children and wait if cancellation encounters unknown effects. The task does not
silently detach unresolved work.

The built-in `text.hash/1` is pure unchanged Hemera hashing. `workspace.read/1`
reads one regular UTF-8 file directly in an explicitly admitted directory via an
opened directory descriptor and no-follow/nonblocking file open on Unix. Nested
paths, symlinks, special files, unsupported platforms, changed optional content
hash and oversized output fail. The returned text is retained verbatim as the
observed result. It is a read capability, not a general shell or filesystem sandbox.
The host/provider remains responsible for its declared wall time/device limits;
this embedded synchronous worker profile claims no hard OS preemption. Adapter
panics are treated as unknown outcomes and do not poison task coordination.

`Adapter`/`ModelProvider` are host interfaces, not capabilities exposed to model
output. `observe` ingests a trusted receipt for one captured operation/attempt;
it can retain that observation after revocation, while engine reconciliation
requires a current management grant. A network receiver must authenticate and
correlate its provider receipt before calling this local API.
