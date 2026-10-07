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

## sixteen organs, two kinds

**chroma** — one component: the eight organs that show, on a 3×3 grid around one pluggable centre. cy designs it; cyb paints it in pixels, a terminal paints it in cells.

```
┌──────────┬──────────┬──────────┐
│   now    │  voice   │   name   │
├──────────┼──────────┼──────────┤
│  sense   │spacetime │  sigma   │
├──────────┼──────────┼──────────┤
│  memory  │   com    │   time   │
└──────────┴──────────┴──────────┘
```

**unseen** — eight organs that work behind the slots, each a component of its own: soma · glia · seer · soul · ward · vault · state · body.

[anatomy](specs/anatomy.md) is the source of truth for every organ.

## what is in this repository

| directory | what |
|---|---|
| [`chroma/`](chroma/README.md) | the chroma as one component: the grid and its cyberlink contract (README), one page per showing organ — now · voice · name · sense · sigma · memory · com · time (log, plan) — and the code of those that have it: [`sigma/`](chroma/sigma/README.md), merged in with its history on 2026-10-07 |
| [`specs/`](specs/README.md) | the agent's contract and anatomy, and the pages of the unseen organs specified here: soma · soul · ward · vault · state · body |

the unseen organs with code keep their repositories: [soma](https://github.com/cyberia-to/soma) (mind), [glia](https://github.com/cyberia-to/glia) (model runtime), [seer](https://github.com/cyberia-to/seer) (proposer), [vault](https://github.com/cyberia-to/vault) (custody). the chroma organs without code yet (com, sense, now, memory, time, voice, name) are implemented today inside [[cyb]] (`cyb/cli` is the `cy` binary, `cyb/core` carries `ChromaId`); they move into `chroma/` as their contracts settle.

## the boundary

one test: cy works over ssh with no display. an organ that needs a camera, a speaker or a rendered world belongs to [[cyb]]; everything else belongs here. the graph, the language and the network are below cy and know nothing of it: the book is [[cybergraph]], what can be said over it is [[neural]], the wire is [[soft3]].

## where cy sits

rung 6 of the ten-product ladder on the front page of [soft3](https://soft3.org), between the language (neural) and the network (soft3): one chip for the chroma, one per unseen organ. the vocabulary is `soft3/specs/terms.md`; this repository does not restate it.

[[cyb]] · [[cybergraph]] · [[neural]] · [[soft3]] · [[cyber]]
