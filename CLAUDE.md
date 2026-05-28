# soma — machine mind

Local cognitive architecture of one cyber Avatar. A machine that perceives, decides, acts, learns, and survives.

## What this is

soma is the agent runtime and cognitive stack for cyb robots. It is a standalone product — the embodied mind layer that sits between raw hardware and the cybergraph.

Each soma instance runs on one Body (physical machine), manages its own resources, earns sigma on the open market, and migrates to a new Body on hardware failure — the Avatar outlasts the hardware.

## Core concepts

- **Avatar** = Name (NFT) + Soul (root Neuron) + Body (current machine)
- **Body** = mortal physical vessel; semcon with resource budget
- **Soul** = root Neuron; holds sigma; orchestrates worker Neurons; immortal
- **Neuron** = cognitive worker; has many Addresses; executes Tasks; holds Skills
- **Sigma** = sum of token balances across all networks; migrates with Soul

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

All inference runs in nox with STARK proof. Provable AI.

## Key dependencies

- **bbg** — append-only memory substrate; polynomial commitment; no locks
- **nox** — STARK VM; every inference is proven
- **cybergraph** — coordination graph; particle + cyberlink storage
- **tri-kernel** — D (diffusion) + S (springs) + H (heat) tri-kernel recomputation
- **zheng** — proof system (~5μs verify)

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

Phase 0 — specification complete (soma.md + soma-spec.md).
Phase 1 — not started. First move: soma-kernel + soma-runtime scaffolding.
