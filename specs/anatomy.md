---
title: cy anatomy
tags: cy, core
alias: cy organs, agent anatomy, the agent's organs
crystal-type: reference
crystal-domain: cyber
---
# anatomy — the organs of the agent

cy is one organism of sixteen organs in two kinds: **eight that show** — the slots of the [[cy/chroma/README|chroma]], the agent's 3×3 surface — and **eight that work unseen** behind them. every page in this repository names things by this table; a page that cannot be traced to an organ is legacy or belongs to [[cyb]], where the body's own organs remain (avatar, brain, vision and the painting doctrine).

## the chroma — what shows

| part | slot | is | today |
|---|---|---|---|
| **[[cy/chroma/now|now]]** | top-left · where | the context — the file the agent stands on, and the hinge of everything: what [[cy/chroma/com|com]] acts on by default, what [[cy/specs/soma|soma]] packs into the model's window, where casts attach, where [[cy/chroma/memory|memory]] stands, the centre [[cy/chroma/time|time]] pivots on | partial — the app tracks a current particle; the organ with its many functions is to build |
| **[[cy/chroma/voice|voice]]** | top-centre · says | what the agent answers: forms a signal and hands it to [[cy/chroma/sense|sense]] to send; not sound | to grow |
| **[[cy/chroma/name|name]]** | top-right · who | the resolver — the agent's name resolves through the graph, owned like a token, not set in a config | ports with the soft3 genesis |
| **[[cy/chroma/sense|sense]]** | mid-left · messages | the messenger — interaction with other neurons, files, robots | seed exists (`money_to_sense`, notices); grows into the inbox/outbox |
| **[[cy/chroma/com|com]]** | bottom-centre · command | the commander — the one input of the agent: a line with completion, on a cli or under a screen; reads [[cy/specs/soul|soul]] on every ask | live (com world in cyb); the prompt of the cy cli |
| **[[cy/chroma/time|time]]** | bottom-right · when | one screen: [[cy/chroma/log|log]] ← now → [[cy/chroma/plan|plan]] — the history of every interaction and the schedule of standing orders, the present in the middle | log live (`~/cyb/graph.log` *is* the log); plan seeded (mining standing order); the view to build |

the centre, **spacetime**, is a slot and not an organ: it hosts whatever renders — the transcript in a terminal, a world in cyb.

## unseen — what thinks

| part | is | today |
|---|---|---|
| **[[cy/specs/soma|soma]]** | mind and model: local inference, weights, silicon | live — soma kernel + honeycrisp, `? q` in com |
| **glia** | the model runtime: local inference over the graph, the engine soma calls | own repository: [glia](https://github.com/cyberia-to/glia) |
| **seer** | the proposer: finds the cyberlink worth making next and hands it to the neuron to sign | own repository: [seer](https://github.com/cyberia-to/seer) |
| **[[cy/specs/soul|soul]]** | the declaration of the mind: one file defining processing when the agent is asked — which model, which dialect, what it may do unasked; soma executes it, ward enforces it | `~/cyb/soul` to define; soma settings are its embryo |
| **[[cy/specs/ward|ward]]** | the enforcer of the soul: the permission boundary every runtime's act passes through — holds the caps, performs or refuses, prompts the neuron | live — the effect router; doctrine at [[cy/specs/ward|ward]] |

## unseen — what it knows

| part | is | today |
|---|---|---|
| **[[cy/specs/state|state]]** | verified perception of external networks: every answer is a value **plus a tier** (T0 proof-verified · T1 anchor-verified · T3 unproven RPC, badged); one query IR, a proof-router, local verification over provider truth | doctrine written — [[cy/specs/state|state]] |

## unseen — what it holds

| part | is | today |
|---|---|---|
| **[[cy/specs/vault|vault]]** | secrets and sleeping neurons: keys, mnemonics, TOTP | live — XChaCha20 under mnemonic + TOTP; sigma spends, vault holds and signs |
| **[[cy/specs/body|body]]** | the physical body: silicon, sensors, energy, mining — telemetry and resources of the machine the robot lives on | live (default world: telemetry, zheng + seer miners, PUSSY/day) |

## banned words

*wallet* → sigma · *face* → avatar (a cyb organ) · *the main config* → soul · *space, ad, ava, brain (as slot names)* → now, voice, name, memory
