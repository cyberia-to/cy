---
title: cy
tags: cy, core
alias: cy agent, the agent, cyberian agent
icon: "🧬"
crystal-type: entity
crystal-domain: cyber
---
# cy

the mind over the book. cy acts for a [[neuron]]: it reads the [[cybergraph]] the neuron keeps, decides, proposes the next [[cyberlink]], speaks it as a [[signal]], holds what the neuron owns and keeps secret what must stay secret. it runs headless anywhere a terminal opens, on a server, a phone or over ssh, and needs no screen. [[cyb]] is cy with a body.

```bash
cargo install cyb   # the cy binary ships with cyb until the code moves here
cy
```

## what cy is made of

sixteen organs. each has a page in [specs](specs/README.md); [anatomy](specs/anatomy.md) is the source of truth.

| kind | organs |
|---|---|
| the [chroma](specs/chroma.md) — eight that show, on a 3×3 grid around one pluggable centre | [[cy/specs/now\|now]] · [[cy/specs/voice\|voice]] · [[cy/specs/name\|name]] · [[cy/specs/sense\|sense]] · [[cy/specs/sigma\|sigma]] · [[cy/specs/memory\|memory]] · [[cy/specs/com\|com]] · [[cy/specs/time\|time]] (log ← now → plan) |
| unseen — eight that work behind the slots | [[cy/specs/soma\|soma]] · glia · seer · [[cy/specs/soul\|soul]] · [[cy/specs/ward\|ward]] · [[cy/specs/vault\|vault]] · [[cy/specs/state\|state]] · [[cy/specs/body\|body]] |

glia (the model runtime) and seer (the proposer) are their own repositories; the other organs are specified here and grow into crates as their contracts settle.

## the boundary

one test: cy works over ssh with no display. a full-screen terminal is a display cy owns — the chroma is its shape in cells as much as in pixels. an organ that needs a camera, a speaker or a rendered world belongs to [[cyb]]; everything else belongs here. the graph, the language and the network are below cy and know nothing of it: the book is [[cybergraph]], what can be said over it is [[neural]], the wire is [[soft3]].

## where cy sits

rung 6 of the ten-product ladder of [soft3](https://soft3.org/layers), between the language (neural) and the network (soft3). the vocabulary is `soft3/specs/terms.md`; this repository does not restate it.

## code

the `cy` toolset (`cy tools`, `cy install`, `cy <tool>`) lives in `cyb/cli` and `cyb/crates/cyb/src/bin/cy.rs` today. it moves here when the specs settle, not before.

[[cyb]] · [[cybergraph]] · [[neural]] · [[soft3]] · [[cyber]]
