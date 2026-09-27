---
title: seer
tags: cyber, cyb, seer, core, warriors
crystal-type: entity
crystal-domain: cyber
icon: "🔮"
alias: seer, cyber-seer, cyber/seer, link miner, mint miner, structure miner
---
# seer

seer is a miner. it finds the [[cyberlink]] worth making next and hands it to the [[neuron]] to sign.

## the problem

the [[cybergraph]] is shared. among its existing [[files]] there are $O(N^2)$ pairs that could be linked. a [[neuron]] has a budget of will and one question: which link, right now, teaches the graph the most for what it costs?

[[tru]] answers that question exactly, before a token is spent. every candidate link has a price: surprise $\rho$ times proven focus shift $\Delta\phi^+$, net of cost ([[rewards]] §6). seer searches the pairs against that price.

## why that is mining

a miner searches a space against a public function and is paid for what it proves. seer does exactly this.

| mining | seer |
|---|---|
| hash function | $\phi^*$, the collective [[focus]] |
| hashrate | proven $\Delta\phi^+$ per epoch |
| difficulty | the surprise gate $\rho$, times the exponential cost of links ([[universal law]]) |
| pool | the graph itself — evidence, not documentation |
| duty cycle | the will the neuron lets it spend |
| block reward | the mint |

the doctrines of [[warriors]] apply as written — make the machine earn, report the true rate. the machine is a neuron's will instead of a chip.

## what it mines

on $CYB a neuron is paid four ways ([[rewards]]):

- the mint — Shapley's share of the proven focus shift $\Delta\phi^+$ its links caused
- the subsidy — proof-of-work for settling other neurons' shares
- query fees
- stake

seer mines the mint. a different miner grinds nonces for the subsidy. two miners, two streams, one graph.

## on any chain

the price is tru's, not $CYB's. any home book that settles in proven focus shift can be mined the same way. and there are many books: [[oikos|oikos]] gives every token its own — one token, one chain — and any neuron with a token can root one. $CYB is the reference, not the only chain. seer mines whichever chain the neuron is signing into.

## never a new file

the price exists only for pairs of files that already exist. a new file's value is the entropy of content nobody has written; it exists after the file is made, not before. nothing prices it in advance.

that is the boundary of the method, not a policy. mining is exact because its space is bounded and every answer is priced ahead. creation is synthesis in an unbounded space, and its risks — hallucination, redundancy, storage paid before worth is known — are the risks of an unpriced search. humans, models, and agents create files. seer wires them in. see [docs/link-production.md](docs/link-production.md).

## when it stops

a pair is worth mining iff $\rho \cdot \Delta\phi^+$ exceeds its cost. when no pair in a neuron's reach clears that bar, seer idles. no threshold to tune — the reward decides, from public state and the neuron's own ego-net. a mined-out graph is one whose structure would not surprise the crowd.

consequences:

- structural work runs out first. bridges raise $\lambda_2$ and spread focus; the mint pays for concentration. [densification](specs/densification.md)'s phases end in maintenance — bostrom's measured $\lambda_2 \approx 0.13$ was already there.
- mining never stops globally. every new file arrives unwired; [[temporal decay]] reopens pairs. mining is the digestion of creation — mined out per region, per epoch, until something new arrives.
- a learned policy trained on realized settlement learns to leave mined-out regions alone on its own.

open:

1. the mint's stop and the network's need are not proven to coincide. [[superadditivity]] measured that connectivity raises the collective advantage $\sigma$ and lowers syntropy $J$; the mint pays for $J$; the subsidy is proof-of-work, not a fund for structure. a graph can be mined out and still under-connected for [[foculus]] finality. either concentration is what matters, or [[rewards]] is missing a structural term. seer reports both numbers — structural score and predicted settlement — so the answer comes from evidence.
2. the loop's fixed point is not proven. tru's contraction converges $\phi^*$ on a fixed graph each epoch. miners changing the graph in response to $\phi^*$ is a different system; whether it settles or oscillates is open ([docs/link-production.md](docs/link-production.md), question 4).

## invariants

1. outside the contraction — seer never computes $\phi^*$ and never touches [[focusing]]. tru stays bit-identical on every machine.
2. the mint is the objective — rank or sample by expected settlement net of cost. structural signals are priors, never the quantity optimized.
3. no new files — edges between files that exist.
4. honesty as a feature — every proposal carries a predicted settlement; every settled epoch scores it against what tru measured. a policy that mispredicts loses trust before it loses stake.

## methods

| method | kind | signal | status |
|---|---|---|---|
| [densification](specs/densification.md) | analytical | Fiedler vector → $\Delta\lambda_2$ per link; bridge → mesh → semantic | specified, unbuilt |
| [learned policy](roadmap/learned-policy.md) | learned | a GFlowNet sampling links in proportion to expected settlement; densification as teacher | proposal |

## map

- [specs/interface.md](specs/interface.md) — the contract: inputs, the mining loop, the output, the invariants
- [specs/densification.md](specs/densification.md) — cyber-seer, the analytical method
- [docs/link-production.md](docs/link-production.md) — why: the intelligence problem, and where mining ends and creation begins
- [roadmap/](roadmap/README.md) — open proposals

## naming

cyber-seer was the densification algorithm when it lived as a research page in [[cyber]]. seer is the component; cyber-seer is its first method.

see [[tru]] for the measure · [[rewards]] for the streams · [[warriors]] for the doctrine · [[cyb]] for the body that runs it
