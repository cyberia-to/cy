---
title: cy anatomy
tags: cy, core
alias: cy organs, agent anatomy, the agent's organs
crystal-type: reference
crystal-domain: cyber
---
# anatomy — the organs of the agent

cy is one organism of sixteen organs. every page in this repository names things by this table; a page that cannot be traced to an organ is legacy or belongs to [[cyb]]. the rows were moved from the robot's anatomy in [[cyb/anatomy|cyb]], where the body's own organs remain (avatar, brain, vision, time and the painting doctrine).

## mind — what thinks

| part | is | today |
|---|---|---|
| **[[cy/specs/soma|soma]]** | mind and model: local inference, weights, silicon | live — soma kernel + honeycrisp, `? q` in com |
| **glia** | the model runtime: local inference over the graph, the engine soma calls | own repository: [glia](https://github.com/cyberia-to/glia) |
| **seer** | the proposer: finds the cyberlink worth making next and hands it to the neuron to sign | own repository: [seer](https://github.com/cyberia-to/seer) |
| **[[cy/specs/soul|soul]]** | the declaration of the mind: one file defining processing when the agent is asked — which model, which dialect, what it may do unasked; soma executes it, ward enforces it | `~/cyb/soul` to define; soma settings are its embryo |
| **[[cy/specs/ward|ward]]** | the enforcer of the soul: the permission boundary every runtime's act passes through — holds the caps, performs or refuses, prompts the neuron | live — the effect router; doctrine at [[cy/specs/ward|ward]] |

## mouth and messenger — what asks and says

| part | is | today |
|---|---|---|
| **[[cy/specs/com|com]]** | the commander — the one input of the agent: a line with completion, on a cli or under a screen; reads [[cy/specs/soul|soul]] on every ask | live (com world in cyb); the prompt of the cy cli; becomes omnipresent chrome in cyb |
| **[[cy/specs/voice|voice]]** | the ability to form a signal and hand it to [[cy/specs/sense|sense]] to send; not sound | to grow |
| **[[cy/specs/sense|sense]]** | the messenger — interaction with other neurons, files, robots | seed exists (`money_to_sense`, notices); grows into the robot's inbox/outbox |

## perception — what it knows

| part | is | today |
|---|---|---|
| **[[cy/specs/state|state]]** | verified perception of external networks: every answer is a value **plus a tier** (T0 proof-verified · T1 anchor-verified · T3 unproven RPC, badged); one query IR, a proof-router, local verification over provider truth | doctrine written — [[cy/specs/state|state]] |
| **[[cy/specs/now|now]]** | the context — the file the robot stands on, and the hinge of everything: what [[cy/specs/com|com]] acts on by default, what [[cy/specs/soma|soma]] packs into the model's window, where casts attach, where [[cyb/parts/brain|brain]] and [[cy/specs/memory|memory]] stand, the center [[cyb/parts/time|time]] pivots on | partial — the app tracks a current particle; the organ with its many functions is to build |

## keeping — what it holds and remembers

| part | is | today |
|---|---|---|
| **[[cy/specs/log|log]]** | the history of every interaction | live: `~/cyb/graph.log` *is* the log; the world is a rune census + numbers table over the chain |
| **[[cy/specs/plan|plan]]** | the schedule: standing orders, deferred intents | seed exists (mining standing order); generalizes |
| **[[cy/specs/memory|memory]]** | the file manager with links: files as a file system, table or tiles; works over a cli as well as a screen | half-live: tap-to-read pages + [[cyb/parts/fs|fs]] become its spec; a projection of the brain in cyb, one key flips brain ⇄ memory |
| **[[cy/specs/sigma|sigma]]** | the sum, in cx by default: a registry of tokens with three measurements each — price, balance, supply — one method per measurement, any rule the owner likes; not a wallet | own repository: [sigma](https://github.com/cyberia-to/sigma); live as a world in cyb |
| **[[cy/specs/vault|vault]]** | secrets and sleeping neurons: keys, mnemonics, TOTP | live — XChaCha20 under mnemonic + TOTP; sigma spends, vault holds and signs |
| **[[cy/specs/body|body]]** | the physical body: silicon, sensors, energy, mining — telemetry and resources of the machine the robot lives on | live (default world: telemetry, zheng + seer miners, PUSSY/day) |

## banned words

*wallet* → sigma · *face* → avatar (a cyb organ) · *the main config* → soul
