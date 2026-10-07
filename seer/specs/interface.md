---
tags: cyber, cyb, seer, spec
crystal-type: spec
crystal-domain: cyber
status: draft
alias: seer interface, seer contract, mining interface, mining loop
---
# interface

the contract between [[seer]], [[tru]], the [[cybergraph]], and the [[neuron]] that signs. seer is a miner: a function from public state and one neuron's ego-net to a set of candidate [[cyberlinks]], each priced by what it would settle for, plus whatever private state a learned policy keeps.

## scope

seer mines existing→existing, on any chain. a chain here is a home book ([[oikos|oikos]]) whose settlement pays for proven focus shift under the [[tru]] measure; $CYB's is the reference and this page reads its rules from [[rewards]], but nothing below is specific to it — a neuron mining its own book substitutes that book's streams. seer proposes edges between [[files]] that are already in the graph and never creates a file. the value of every candidate is computable before commitment from public state ([[rewards]] §6); the value of a new file is not — it exists only once the file is wired in. that asymmetry is the boundary, and it is why a content-blind miner can be exact at its job. see [docs/link-production.md](../docs/link-production.md).

## inputs

everything seer reads is public, per-epoch, and already computed by someone else.

| input | from | what it is |
|---|---|---|
| $\phi^*$, cyberank | [[focusing]] | the collective focus distribution and its per-particle reading |
| spectral positions $V_k$ | [[focusing]] | the bottom-$k$ Laplacian eigenvectors, Fiedler vector $v_2$ first; row $i$ is particle $i$'s coordinate |
| $\lambda_2$, $\kappa$ | [[focusing]] | algebraic connectivity and the contraction, per epoch |
| syntropy $J$ | [[focusing]] | how concentrated $\phi^*$ is |
| $\Delta\phi^+_\nu(S)$ | [[rewards]] §6, propose phase | the neuron's standalone directed impulse for a candidate set $S$ — the exact locality-bounded recompute on its $O(\log 1/\varepsilon)$-hop neighbourhood, in the fixed $T(\varepsilon)$ steps of [[arithmetic]] |
| $\rho$ | [[rewards]] §5, [[truth-scoring]] | the surprise gate — what the crowd's predictions already expect |
| signals, cyberlinks, particles | [[cybergraph]] | the graph, and the neuron's own ego-net |
| will, conviction | the neuron ([[cyb]]) | what the neuron can spend, and where it already stands |

seer reads the Fiedler vector; it does not run an eigensolver. tru emits $V_k$ every epoch, and [densification](densification.md) needs $v_2$ at tru's cadence, not faster.

## the mining loop

```
every epoch:
  read φ*, V_k, λ₂, J from tru; the ego-net and the budget of will from the neuron
  search the pair space for candidates            (densification, a learned policy, or both)
  price each candidate:
      settlement  ŝ = ρ · Δφ⁺_ν({link})           the ceiling among substitutes, rewards §6
      cost        c = will + conviction spent
  keep candidates with ŝ > c; rank or sample by ŝ − c
  emit the proposal to the neuron
  the neuron signs what it wants; the signal goes out
  at settlement tru measures the realized share; score ŝ against it
```

the difficulty is not a target hash; it is the product of the surprise gate and the exponential cost of links ([[universal law]]). the block reward is Shapley's share of the realized $\Delta\phi^+$. the hashrate is proven $\Delta\phi^+$ per epoch.

## output

a proposal: a ranked or sampled set of candidate cyberlinks $(p, q, \tau, a, v)$ for one neuron, each carrying

- the predicted settlement $\hat{s}$
- the structural score — $\Delta\lambda_2$, articulation bypass, or focus redistribution, per the method's phase
- the cost
- the method and its confidence

and, per epoch, a report: predicted against realized settlement, structural score against realized $\Delta\lambda_2$, will spent, and whether the neuron's reach is mined out. seer publishes the report as a [[cyberlink]] into the graph it mines.

the neuron decides. seer proposes; the owner signs through [[cy/specs/ward|ward]] and the vault in cyb, or an agent's own key; tru measures at settlement. the realized value is the training signal for any learned policy.

## when the loop idles

a candidate is worth mining iff $\hat{s} > c$. when no pair in the neuron's reach clears that, seer emits an empty proposal and the report says so. this is the stop condition — built into the reward, not a threshold seer sets. structural work ends first (bridges spread focus; the mint pays for concentration), semantic work last, and a created file or a decayed link reopens the region. the open questions — whether the mint's stop matches the network's need, and whether miners plus focus have a fixed point — are stated on the [README](../README.md); seer's contribution to them is to report both numbers honestly.

## invariants

1. outside the contraction. seer never computes $\phi^*$, never participates in [[focusing]], never proposes a change to the iteration. tru's determinism is what [[zheng]] proves.
2. the mint is the objective. a method ranks or samples by expected settlement under [[rewards]] net of cost. structural signals shape the search; they are never the quantity optimized. the protocol pays for directed syntropy that is valid and surprising, and a copy of an obvious link scores zero however large its $\Delta\lambda_2$.
3. no new files.
4. public aggregates only. individual cyberlinks are private to their neurons; $\phi^*$, axon weights, and spectral positions are public. seer reads the public side and its own neuron's ego-net, never another neuron's. a learned policy therefore optimizes publicly visible focus flow, which is the right objective.
5. proposal, then commitment. a method may be cheap and wrong; the neuron pays only when it signs. the prediction is scored against the realized settlement, and a policy that mispredicts loses the neuron's trust before it loses the neuron's stake.
6. provable when it matters. a proposal produced by a [[nox]] program carries a [[zheng]] trace: this policy, on this public state, produced these candidates. that proves the process; settlement proves the quality.

## what the structural score is worth

densification's early phases raise $\lambda_2$ — bridges and bypasses that spread focus. [[superadditivity]] measured that connectivity raises the collective's advantage $\sigma$ and lowers syntropy $J$; the mint pays for directed syntropy, and the subsidy stream is stake-blind proof-of-work for settlement, not a fund for structure. so a bridge can be what the network needs and settle for little. seer reports the structural score and the predicted settlement as two numbers and never pretends the first is the second.

see [densification](densification.md) for the first method · [[rewards]] for the four streams · [[impulse]] for the quantity measured · [[warriors]] for the doctrine
