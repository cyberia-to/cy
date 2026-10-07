---
title: cy
tags: cy, core
alias: cy agent, the agent, cyberian agent
icon: "🧬"
crystal-type: entity
crystal-domain: cyber
---
# cy

the mind over the book. cy acts for a [[neuron]]: it reads the [[cybergraph]] the neuron keeps, decides, proposes the next [[cyberlink]], speaks it as a [[signal]], holds what the neuron owns and keeps secret what must stay secret. it runs headless anywhere a terminal opens — a server, a phone, over ssh — and needs no window: a full-screen terminal is its display. [[cyb]] is cy with a body.

## the shape: one chroma

cy is sixteen organs in one shape — the [chroma](specs/chroma.md), a 3×3 grid of the eight organs that show, around one pluggable centre, with eight organs working unseen behind them. cy designs the chroma; cyb paints it in pixels, a terminal paints it in cells.

```
┌──────────┬──────────┬──────────┐
│   now    │  voice   │   name   │
├──────────┼──────────┼──────────┤
│  sense   │spacetime │  sigma   │
├──────────┼──────────┼──────────┤
│  memory  │   com    │   time   │
└──────────┴──────────┴──────────┘
        unseen: soma · glia · seer · soul · ward · vault · state · body
```

[anatomy](specs/anatomy.md) is the source of truth for every organ; one page per organ in [specs](specs/README.md).

## what is in this repository

the organs that had repositories of their own live here since 2026-10-07, each as a directory with its full history (`git subtree`); the source repositories are to be archived.

| directory | organ | what it is | build |
|---|---|---|---|
| [`specs/`](specs/README.md) | all | the contracts: agent, anatomy, chroma, one page per organ | — |
| [`soma/`](soma/README.md) | soma (unseen — mind) | the mind and its model: `agent/` (the neuron's runtime loop), `kernel/` (local inference over honeycrisp, streams tokens to the host), `specs/`, `audit/`, `research/` | `cd soma/agent && cargo check` · `cd soma/kernel && cargo check` |
| [`glia/`](glia/README.md) | glia (unseen — model runtime) | the engine soma calls: `run/` (inference runtime), `import/` (model import), tiered tests | `cd glia && cargo test` |
| [`seer/`](seer/README.md) | seer (unseen — proposer) | finds the cyberlink worth making next and hands it to the neuron to sign; specs, docs, roadmap — no code yet | — |
| [`sigma/`](sigma/README.md) | sigma (chroma — the sum) | a registry of tokens with three measurements each — price, balance, supply; cx is the default measure; specs — the TUI lives in `~/sigma` until it moves | — |
| [`vault/`](vault/README.md) | vault (unseen — keeping) | secrets and sleeping neurons: encrypted custody, signs native neuron actions, never casts to the graph | `cd vault && cargo test` |

each organ keeps its own `Cargo.toml` / workspace; there is no root workspace on purpose — organs build independently, and their outward `path` dependencies (`../../hemera/rs`, `../../neuron/…`, `../../mudra`) point one level up to the sibling repositories under `~/cyber`.

the remaining organs — com, voice, name, sense, now, memory, time (log · plan), soul, ward, state, body — are specified here and implemented today inside [[cyb]] (`cyb/cli` is the `cy` binary, `cyb/core` carries `ChromaId`); they move here as their contracts settle, not before.

## the boundary

one test: cy works over ssh with no display. an organ that needs a camera, a speaker or a rendered world belongs to [[cyb]]; everything else belongs here. the graph, the language and the network are below cy and know nothing of it: the book is [[cybergraph]], what can be said over it is [[neural]], the wire is [[soft3]].

## where cy sits

rung 6 of the ten-product ladder of [soft3](https://soft3.org/layers), between the language (neural) and the network (soft3). the vocabulary is `soft3/specs/terms.md`; this repository does not restate it.

[[cyb]] · [[cybergraph]] · [[neural]] · [[soft3]] · [[cyber]]
