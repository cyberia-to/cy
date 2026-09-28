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

| group | organs |
|---|---|
| mind | [[cy/specs/soma\|soma]] · glia · seer · [[cy/specs/soul\|soul]] · [[cy/specs/ward\|ward]] |
| mouth and messenger | [[cy/specs/com\|com]] · [[cy/specs/voice\|voice]] · [[cy/specs/sense\|sense]] |
| perception | [[cy/specs/state\|state]] · [[cy/specs/now\|now]] |
| keeping | [[cy/specs/log\|log]] · [[cy/specs/plan\|plan]] · [[cy/specs/memory\|memory]] · [[cy/specs/sigma\|sigma]] · [[cy/specs/vault\|vault]] · [[cy/specs/body\|body]] |

glia (the model runtime) and seer (the proposer) are their own repositories; the other organs are specified here and grow into crates as their contracts settle.

## the boundary

one test: cy works over ssh with no display. an organ that needs a screen, a camera or a speaker belongs to [[cyb]]; everything else belongs here. the graph, the language and the network are below cy and know nothing of it: the book is [[cybergraph]], what can be said over it is [[neural]], the wire is [[soft3]].

## where cy sits

rung 6 of the ten-product ladder of [soft3](https://soft3.org/layers), between the language (neural) and the network (soft3). the registry is `soft3/release/components.toml`, the vocabulary `soft3/specs/terms.md`; this repository does not restate them.

## code

the `cy` toolset (`cy tools`, `cy install`, `cy <tool>`) lives in `cyb/cli` and `cyb/crates/cyb/src/bin/cy.rs` today. it moves here when the specs settle, not before.

[[cyb]] · [[cybergraph]] · [[neural]] · [[soft3]] · [[cyber]]
