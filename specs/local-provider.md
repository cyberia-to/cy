# Local model provider for durable tasks

Status: accepted initial adapter contract, separate from remote/provider parity.

The kernel supplies `glia/local-sha256/1` to Soma's model-provider interface.
A trusted host pins the absolute model path and a revision:
H(`soma/glia-model/sha256/1` || SHA256(exact file bytes) || byte length LE64).
This is an explicitly named weight-file checksum profile; native neuron IDs,
particle hashing and protocol signatures do not change. Before and after loading
new weights the provider verifies this revision. Model artifacts must remain
stable during loading. A loaded model is kept only for the exact admitted revision;
subsequent path replacement cannot change already loaded weights.

Each request pins maximum generated tokens, sampling parameters, prompt bound,
wall-time ceiling and model revision. Generated output is bounded by the existing
64 KiB task result cap. Prefill and decode check a cooperative deadline between
model steps. A GPU/kernel call itself is not hard-preemptible in this local
profile. No token or instruction count is called a payment or proof.

The model receives host-authored instructions separately from user input, scoped
memory and verbatim tool observations. JSON-structured final/tool/delegation
responses are validated against the task schemas. Ordinary text is a final answer;
a malformed response that attempts structured output is an observed failure.
Tool names, schemas, source scopes, signatures and grants never come from the
model as authority. The provider holds no keys.

A bounded local queue is the provider handoff. Enqueue occurs only inside a fresh
neuron worker claim and current attachment guard; it admits that exact pinned
request to the body. Already accepted work may finish after cancellation or
revocation. New work needs a new claim. Events carry task/operation/attempt and
are observations, not a permit to call again. Dropping/restarting the queue loses
no claimed permission: its unresolved operations remain unknown in neuron and
must not be automatically re-enqueued. A trusted host persists each observed
result through Soma before attempting engine reconciliation. Streaming deltas
are provisional presentation; the terminal observation is authoritative.

The queue holds at most four pending model requests and the event channel at most
64 events. Saturation refuses a new request as a known local admission failure.
The receiver must keep draining events (including for cancelled tasks) so the
bounded producer does not deadlock. Application controls are accepted only before
a terminal application observation; after that, steering/context change requires
follow-up work rather than silently discarding a purportedly accepted control.

The real-weight integration is selected explicitly with
`cargo test --features model-integration --test durable_provider` in kernel/.
`SOMA_TEST_MODEL` can name the local artifact; otherwise it uses the configured
Soma model. Selecting this test without available weights fails, never silently
passes or downloads weights. Ordinary kernel unit tests need no new model asset.
The development profile optimizes SHA-256 file verification; this affects speed,
not the revision bytes.
