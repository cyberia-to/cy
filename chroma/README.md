---
title: chroma
tags: cy, core, architecture
alias: chroma, the nine, cy chroma, agent interface
crystal-type: spec
crystal-domain: cyber
---
# chroma — the shape of the agent, one component

chroma is one component of cy: the eight organs that show, their contracts (one page each in this directory) and the code of those that have it (`sigma/`). cy's interface is a **3×3 grid**: eight organs that show, arranged around one pluggable centre. each slot owns a fixed position, a single role and a strictly typed [[cyberlink]] interface. slots never share state — every interaction crosses a typed cyberlink. the grid is closed: the nine slots are the entire surface of the agent.

the shape is cy's, not the screen's. a terminal is a full-screen surface too — a modern coding agent is exactly this grid in cells: an input line, a transcript, a status bar, a plan panel, notifications, cost. cy *designs* the chroma; [[cyb]] *paints* it with pixels and worlds ([[prysm]] maps one atom to terminal cells, pixels or world units). one shape, two renderers — the interface analogue of write once, prove anywhere.

## the grid

```
┌──────────┬──────────┬──────────┐
│   now    │  voice   │   name   │
│  where   │   says   │   who    │
├──────────┼──────────┼──────────┤
│  sense   │          │  sigma   │
│ messages │spacetime │ the sum  │
├──────────┼──────────┼──────────┤
│  memory  │   com    │   time   │
│   map    │ command  │ log·plan │
└──────────┴──────────┴──────────┘
```

axes:
- **vertical**: identity (top) ↔ action (bottom)
- **horizontal**: location / perception (left) ↔ value / identity (right)
- **centre**: spacetime, the only pluggable slot

## the eight that show

| slot | position | organ | role |
|---|---|---|---|
| [[cy/specs/now\|now]] | top-left | now | where — the file the agent stands on; the hinge every other organ pivots on |
| [[cy/specs/voice\|voice]] | top-centre | voice | says — what the agent answers; [[cy/specs/soma\|soma]] thinks behind it, unseen |
| [[cy/specs/name\|name]] | top-right | name | who — the neuron's name, resolved through the graph; cyb draws it as the avatar |
| [[cy/specs/sense\|sense]] | mid-left | sense | messages — other neurons, files, robots; the inbox and outbox |
| [[cy/specs/sigma\|sigma]] | mid-right | sigma | the sum — tokens, balances, the body's energy and work as rows of one registry; contract and code in [`sigma/`](sigma/README.md) |
| [[cy/specs/memory\|memory]] | bottom-left | memory | map — files with links, as a table, tiles or tree; cyb draws it as the brain |
| [[cy/specs/com\|com]] | bottom-centre | com | command — the one input; keyboard navigation is context-sensitive across all slots |
| [[cy/specs/time\|time]] | bottom-right | time | log ← now → plan: what it did, what it will do, the present in the middle |

**spacetime** (centre) hosts exactly one renderer at a time. in a terminal it is the transcript or the file under now; in cyb it is a world — mir, a terminal canvas, web content. the eight slots are agnostic to what is in the centre; they learn its identity through now.

## the eight that work unseen

[[cy/specs/soma|soma]] · glia · seer · [[cy/specs/soul|soul]] · [[cy/specs/ward|ward]] · [[cy/specs/vault|vault]] · [[cy/specs/state|state]] · [[cy/specs/body|body]] have no slot. they stand behind one: soma answers into voice, ward gates what com may do, vault signs what sigma spends, state grades what sense and sigma show, body's telemetry is rows of sigma. [[cy/specs/anatomy|anatomy]] keeps the full table; the unseen organs are components of their own — soma, glia, seer and vault have repositories, soul, ward, state and body are specified in `specs/`.

## the cyberlink interface

slots communicate **only** via cyberlinks — the same primitive the [[cybergraph]] uses on-chain. direct reads across slots are forbidden.

a cyberlink is a 5-tuple across three layers:

```
ℓ = (from, to, token, amount, valence)
```

| field | type | layer | meaning |
|---|---|---|---|
| `from` | Particle | structural | the particle of the sending slot |
| `to` | Particle | structural | the particle of the target slot |
| `token` | Particle | economic | what moves — the intent type as a particle |
| `amount` | u64 | economic | weight of this link (1 for most interface messages) |
| `valence` | i8 {-1,0,+1} | epistemic | confirm / neutral / refute |

each slot has a particle identity. intents are particles too — adding an intent means minting a particle, not changing code. when one action must emit several cyberlinks atomically they travel as one [[signal]]; local slot-to-slot signals carry no proof and become provable when the neuron signs and broadcasts.

| from | to | token | meaning |
|---|---|---|---|
| com | spacetime | submit | the user submitted text |
| com | spacetime | switch-renderer | replace the active renderer |
| voice | time | said | an answer lands in the log |
| sense | now | arrived | a message names a file; now may move |

## where the painting lives

the renderer mapping — pixels, cameras, overlay order, the per-slot visual contracts — is [[prysm]]'s: `prysm/chroma/specs/`. in code the slots are `cyb_core::ChromaId` (`cyb/core/src/chroma.rs`), named by these organs since 2026-10-07, with `from_painter_name` reading the old names (space, ad, ava, brain) off old particles; `WorldState::slot()` in the shell is the worlds-as-slots table the overlay drives. the overlay itself — nine slots on one screen, in cells for cy and in pixels for cyb — is the open piece: today the shell switches whole worlds and the cy cli is a flat command list.

[[cy]] · [[cy/specs/anatomy|anatomy]] · [[cyb]] · [[prysm]]
