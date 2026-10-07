# soma — machine mind

Local cognitive architecture of one cyber Avatar. A machine that perceives, decides, acts, learns, and survives.

## What this is

soma is the agent runtime and cognitive stack for cyb robots. It is a standalone product — the embodied mind layer that sits between raw hardware and the cybergraph.

Each soma instance runs on one Body (physical machine), manages its own resources, earns sigma on the open market, and migrates to a new Body on hardware failure — the Avatar outlasts the hardware.

## Core concepts

The accepted cyb foundation ([architecture](../cyb/specs/architecture.md),
[neuron](../cyb/specs/neuron.md)) takes precedence over older Soma drafts.

- **Robot** = named product with explicit neuron attachments; no root signer.
- **Avatar** = a presentation of the robot or a specified subject.
- **Body** = devices, workers and resources; one body can host many neurons and
  one neuron can use several devices. Placement creates no identity.
- **Soul** = versioned configuration, pinned for admitted work; not a neuron.
- **Neuron** = native/foreign domain-qualified subject and its authorized progs.
  Native key IDs and existing foreign addresses do not change in this migration.
- **Task** = Soma work bound to a prog/invocation, context and budget; no task key.
- **Sigma** = scoped holdings and observations across selected attachments;
  unrelated network balances are not interchangeable or implicit finality.

## The four loops

soma runs four concurrent loops, each at its own timescale:

| loop | what it does |
|---|---|
| perception-action | predict → compare → act-or-update (active inference) |
| homeostasis | forecast resource deficits, pre-buy before crash |
| attention | DMN/TPN salience switching (task vs consolidation) |
| market | scan opportunities, trade, grow sigma |

## Work grammar — five primitives

```
WHAT      Goal ↔ Task      what we want / what we do
HOW       Skill            how we are able
WHEN      Event            when things happen
PERCEIVE  Sensor           what wakes us
```

Everything reduces to configurations of these five + typed cyberlinks.

## Model stack

19 models across 4 tiers. Intelligence lives in memory, not weights.

- **Tier 0** (~1.5GB, always-on): 8 small specialists for routing, embedding, urgency, language, intent, anomaly, splitting, injection-check
- **Tier 1** (<2s load): bitnet-2B + qwen3.5-4b + nuextract + qwen2.5-coder-1.5b
- **Tier 2** (<6s load): qwen3.5-9b + qwen2.5-coder-14b + mimo-7b + deepseek-r1-8b + qwen2.5-vl-7b
- **Tier 3** (external): Anthropic API + Perplexity (irreversible decisions only, <5%)

The tier list is a target inventory. The current kernel calls glia's local model
runner. A proof is claimed only for a pinned, implemented and verified profile;
ordinary local inference does not become proven by changing the actor name.

## Key dependencies

- **bbg** — common transactional persistence and authenticated state; local writers share its coordinator
- **nox** — VM/provable execution profiles where supported
- **cybergraph** — coordination graph; particle + cyberlink storage
- **tri-kernel** — D (diffusion) + S (springs) + H (heat) tri-kernel recomputation
- **zheng** — proof system; performance and support come from owner evidence

## Files

- `soma.md` — product overview (for the curious/general audience)
- `soma-spec.md` — complete technical specification (nine layers: Identity, Resources, Survival, Perception, Cognition, Work, Coordination, Memory, Build)
- `research/agent-research.md` — survey of 20 agent/sandbox/runtime projects; what cyb can lift
- `research/hermes-learning-loop.md` — Hermes (Nous Research) learning loop deep-dive; four-cycle topology

## Build strategy (derived from research)

Crate decomposition (target):

```
soma-kernel      orchestration, scheduling, RBAC
  ├─ soma-runtime     agent loop, tool registry, WASM/JS sandbox
  ├─ soma-providers   rig-based LLM layer (20+ providers)
  ├─ soma-memory      procedural (skills) + semantic (graph) + user model
  ├─ soma-channels    Telegram / Discord / Web / TUI / Bevy adapters
  ├─ soma-tools       MCP-exposed tool catalog; WIT interfaces
  ├─ soma-skills      agentskills.io portable skills, auto-curated
  ├─ soma-fs          SQLite-backed agent workspace (agentfs pattern)
  ├─ soma-trust       secret substitution, capability profiles, audit
  ├─ soma-api         MCP server + ACP + A2A endpoints
  └─ soma-hands       autonomous always-on capabilities (cron-driven)
```

## Status

The local cognition kernel and Bevy bridge exist in `kernel/` and `cyb/shell`.
Soma persists task/context/operation observations through neuron progs and Rune.
The shared cyb Host connects both cy task and Bevy; the host owns publication
under a captured attachment. Durable task/worker composition is tracked by P09 in
[neuron-cell convergence](../soft3/roadmap/neuron-cell-convergence.md).
Implementation evidence belongs in audit; older research/provider lists are not
release capability claims. New task schemas must use neuron/prog/invocation
identity and preserve context, grant, budget, schedule and parent-child recovery.
