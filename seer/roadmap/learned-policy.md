---
tags: cyber, cyb, seer, roadmap
crystal-type: process
crystal-domain: cyber
status: proposal
alias: learned policy, learned mining policy, learned proposal, gflownet focus flow, gflownet proposal, learned link proposal, GFlowNet, gflownet
---
# learned policy

formal proposal. a learned mining policy for [[seer]]: a Generative Flow Network that samples candidate [[cyberlinks]] in proportion to what they would settle for, with [densification](../specs/densification.md) as its analytical teacher.

## 0. the invariants this proposal must not break

- the policy lives on the neuron's side, outside the [[focusing]] contraction. it never computes $\phi^*$ and never changes how tru does.
- the reward is the mint. the policy samples in proportion to the neuron's expected settlement under [[rewards]] net of cost. no second score for tru to honor.
- no new files. the action space is pairs of existing files and conviction on existing axons.
- public aggregates only ([interface](../specs/interface.md), invariant 4).

## 1. the problem

a [[neuron]] mining the mint faces a combinatorial search: which [[files]] to connect, with what conviction, to earn the most from a finite budget of will. densification searches this space analytically and deterministically. the question is whether a learned model can do better — propose a distribution over good edits in proportion to their value, rather than the single best one — and keep improving from what actually settled.

## 2. why GFlowNets

a GFlowNet (Bengio et al. 2021) constructs structured objects by sequential actions and produces samples with probability proportional to a reward:

$$p_\theta(x) \propto R(x)$$

reinforcement learning finds the mode — the one best action — which for a graph means monoculture: every miner proposes the same link, and surprise $\rho$ goes to zero for all of them. a GFlowNet samples the distribution, so proposals stay diverse in proportion to value, which is exactly what a surprise-gated reward pays for. MCMC also samples the distribution, but a GFlowNet is amortized: once trained, one sample is one forward pass, no mixing time. the training objective is trajectory balance (Malkin et al. 2022):

$$\log \frac{Z \prod_t p_F(s_{t+1} \mid s_t)}{\prod_t p_B(s_t \mid s_{t+1}) \, R(x)} = 0$$

with forward policy $p_F$, backward policy $p_B$, partition function $Z$; credit propagates over full trajectories via sub-trajectory balance.

## 3. the reward

the reward is what the neuron is paid, read from [[rewards]]:

$$R(x) = \exp\Big(\beta \big[\, \rho(x)\,\Delta\phi^+_\nu(x) \;-\; c(x) \;+\; u(x) \,\big]\Big)$$

- $\Delta\phi^+_\nu(x)$ — the neuron's standalone directed impulse for the edit, [[rewards]] §6 propose phase: exact, locality-bounded, fixed $T(\varepsilon)$ steps. the ceiling of what settles among substitutes.
- $\rho(x)$ — surprise, [[rewards]] §5: the BTS gate that pays nothing for a copy however large its impulse. the protocol's novelty term, not a design choice here.
- $c(x)$ — will and conviction the edit spends.
- $u(x)$ — the neuron's private utility. outside the protocol; the one term the neuron sets.

the exponential form is the optimal proposal distribution under a finite budget ([[universal law]]). densification's signals are not in $R$; they enter as the teacher (§4) and as priors that shape the search.

## 4. densification as teacher

| | densification | learned policy |
|---|---|---|
| approach | analytical: Fiedler vector, articulation points, focus hubs | learned: policy network, trajectory balance |
| signal | $\Delta\lambda_2$, resilience, $\Delta J$ | expected settlement |
| output | ranked top-$K$ | distribution over candidates |
| diversity | deterministic | stochastic, proportional to value |
| phases | explicit, from $\lambda_2$ | emergent from the cost and surprise terms |
| cost | cheap | training plus inference |

they compose. densification's Fiedler-optimal links pre-train the policy by behavioural cloning; the policy then generalizes past the spectral signal — semantic shortcuts, multi-hop bridges, links that raise $\Delta\phi^+$ in ways the Fiedler vector does not predict. densification's phases emerge in the learned policy on their own: early, structural links have the highest settlement per cost; late, the exponential cost and the surprise gate leave only high-impulse semantic links. the policy is never told which phase it is in, and it learns to idle when a region is mined out, because a mined-out region is one where every candidate's $\rho \cdot \Delta\phi^+$ falls below its cost.

one caution from [[superadditivity]]: bridges raise $\lambda_2$ and lower syntropy, and the mint pays for directed syntropy. a teacher that only knows bridges teaches links that settle for little. cloning should use densification's semantic-phase proposals as much as its bridge-phase ones; settlement decides which lessons survive.

## 5. the loop

```
1. snapshot φ*, V_k, λ₂ from tru; the neuron's ego-net from the graph
2. the policy samples a batch of candidate edits
   (add a cyberlink, raise conviction on an axon, attach evidence)
3. price each: ρ · Δφ⁺_ν(x) via the propose-phase marginal — or the surrogate (§6, Q1)
4. keep candidates above cost; the neuron signs the subset it wants
5. tru measures the realized share at settlement
6. train the policy on realized settlement
7. repeat
```

## 6. open questions

### Q1 — a surrogate for the marginal

the exact per-candidate value exists: [[rewards]] §6 defines the standalone marginal as a locality-bounded recompute in fixed steps, and [[impulse]] is the same quantity measured. what the exact recompute cannot do is price thousands of candidates per step on a phone. the open question is a surrogate: a GNN trained on realized $\Delta\phi^+$ over local subgraphs, $O(1)$ per evaluation; or personalized push-back updates, $O(1/\varepsilon)$ per single-edge change; or a low-rank spectral update. the policy is only as good as this surrogate — a poor one mines noise. the exact marginal stays the price of record for whatever the neuron signs.

### Q2 — action spaces of $10^6$ and beyond

published GFlowNets construct graphs of tens of nodes with thousands of actions per step; the cybergraph has billions of particles. three reductions: hierarchical actions (namespace first, particle second, $O(\sqrt N)$ per level); focus-guided masking (only particles with $\phi^* > \varepsilon$ as targets — [[universal law]] says most focus sits on a small fraction); and per-neuron policies over each neuron's own context, the global effect emerging from many local miners. the third matches the architecture. the org has an in-house GFlowNet trainer, built for compiler cost optimization in [[trisha]]; a starting point for the training loop, not for the action space.

### Q3 — privacy

training needs to see which edits improved $\phi^*$; individual cyberlinks are private. the resolution is invariant 4: the policy reads public aggregates — $\phi^*$, axon weights, spectral positions — and its own neuron's ego-net, and the neuron decides privately what to sign. the policy cannot optimize for another neuron's private pattern, which is the right restriction.

### Q4 — provability

a policy that is a [[nox]] program produces a [[zheng]] trace: this policy, on this public state, produced these candidates. that proves the process, not the quality — quality comes from settlement. the harder question is whether training can be proved: gradient steps on the trajectory-balance loss as a nox program, giving verified model updates. open.

## 7. honest assessment

| aspect | status | confidence |
|---|---|---|
| GFlowNet theory | mature, peer-reviewed | high |
| GFlowNets for graph construction | demonstrated (DAG-GFlowNet) | high |
| at $10^6$+ actions | undemonstrated | low |
| surrogate for $\Delta\phi^+$ | research question | medium |
| privacy-compatible training | feasible on public aggregates | medium |
| provable proposals | architecturally possible | medium |
| provable training | open | low |
| the measure it needs | [[tru]] focusing, impulse, rewards — built | — |

the direction is valid and the architecture is compatible. what changed since the first draft is the dependency: the measure this policy learns from is built. what it still needs is settlement running on a live graph, so realized $\Delta\phi^+$ exists to train on.

## 8. what to do first

1. the surrogate, as its own question — any miner needs a fast price for the marginal; worth solving regardless of GFlowNets
2. a synthetic prototype — $10^4$ particles with known $\phi^*$; does the policy learn to sample high-$\Delta\phi^+$ links, does it idle on a mined-out region, and how does diversity compare to random and greedy?
3. integrate when settlement is live — train on realized values from a real graph, not on the surrogate

## references

1. E. Bengio et al., Flow Network based Generative Models for Non-Iterative Diverse Candidate Generation, NeurIPS 2021
2. N. Malkin et al., Trajectory Balance: Improved Credit Assignment in GFlowNets, NeurIPS 2022
3. T. Deleu et al., DAG-GFlowNet: Bayesian Structure Learning with GFlowNets, ICML 2022
4. N. Malkin et al., GFlowNets and Variational Inference, ICLR 2023
5. E. Bengio et al., GFlowNet Foundations, JMLR 2023

see [densification](../specs/densification.md) for the teacher · [[rewards]] for the reward · [[impulse]] for the measured quantity · [[collective focus theorem]] for why exponential proposals are optimal
