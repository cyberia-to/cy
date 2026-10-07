---
tags: cyber, cyb, seer, spec
crystal-type: process
crystal-domain: cyber
status: draft
alias: densification, cyber-seer densification, link densification, spectral densification
---
# densification

cyber-seer: where to link when links cost more every time. the analytical mining method of [[seer]] — maximize spectral gap gained per unit of cost, pushing the graph through its phase transition before links become prohibitively expensive. it mines structure only: pairs of files that exist, never a new file.

## the cost

the linking cost is exponential in supply ([[universal law]]):

$$c(n) = c_0 \cdot e^{\lambda n}$$

where $n$ is the total [[cyberlinks]] in the [[cybergraph]]. the first million links are orders of magnitude cheaper than the second million. every link spent on noise is a link not spent on structure, and the penalty compounds.

## the metric

the spectral gap $\lambda_2$ of the graph [[Laplacian]] controls [[foculus]] finality, [[tri-kernel]] convergence, and the collective advantage $\sigma$ ([[superadditivity]]). adding a link $(i, j)$ changes it by

$$\Delta\lambda_2(i,j) \approx (v_2(i) - v_2(j))^2$$

where $v_2$ is the Fiedler vector. particles on opposite sides of the weakest cut carry opposite signs in $v_2$; linking them bridges the cut. seer reads $v_2$ from [[focusing]]'s spectral positions each epoch ([interface](interface.md)) — it does not compute it.

with constant cost, maximize $\Delta\lambda_2$ per link. with exponential cost, maximize the ratio:

$$\text{ROI}(i,j) = \frac{\Delta\lambda_2(i,j)}{c(n)}$$

the denominator grows exponentially and the numerator is bounded. so early links have astronomical ROI (bridges), mid links moderate (mesh), late links tiny (only semantic precision survives). the strategy shifts from structural to semantic as cost grows.

## three phases

### bridges — cost below 10× base

1. take $v_2$ from the current epoch
2. find unlinked pairs $(i,j)$ with maximal $|v_2(i) - v_2(j)|$
3. rank by $\Delta\lambda_2 / c(n)$; exclude pairs inside one dense cluster
4. propose the top-$K$ per budget cycle
5. re-read $v_2$ next epoch — the cut shifts as bridges form

target: connect components. each bridge moves $\lambda_2$ by $O(1/|P|)$; $K$ bridges closing one bottleneck move it by orders of magnitude. metric: $\lambda_2$ growth per link spent.

### mesh — cost between 10× and 100× base

the graph is connected but thin.

1. find [[articulation points]] — particles whose removal disconnects the graph
2. for each, the two subgraphs it joins
3. propose bypass links between those subgraphs, around the articulation point
4. prioritize by component size: severing a large component costs more

secondary: within dense clusters, reinforce the sparsest inter-cluster paths; the heat kernel at medium $\tau$ ([[tri-kernel]] §1.3) exposes structure at that scale. target: no single point of failure. metric: algebraic connectivity per biconnected component.

### semantic — cost above 100× base

quantity fails; quality remains.

1. read $\phi^*$
2. find high-$\phi^*$ particles with low out-degree — hubs that point nowhere
3. find low-$\phi^*$ files with high intrinsic value — large content, diverse inbound neurons
4. propose links from hubs to undervalued files — this redistributes focus mass toward what deserves it
5. rank by syntropy gained per cost, $\Delta J / c(n)$

target: focus reflects truth. metric: syntropy per link. this is the phase where the structural score and the settlement agree: concentrating focus is what the mint pays for ([interface](interface.md), the tension).

## budget across phases

given a budget $B$ of links before cost becomes prohibitive, the split follows the current $\lambda_2$:

| $\lambda_2$ | phase | allocation | what to link |
|---|---|---|---|
| < 0.001 | bridges | 70% bridges | Fiedler-maximal pairs |
| 0.001 – 0.01 | mesh | 50% bridge · 30% mesh · 20% semantic | articulation bypasses |
| 0.01 – 0.1 | semantic | 20% mesh · 80% semantic | focus-redistributing links |
| > 0.1 | maintenance | 100% semantic | truth refinement |

[[bostrom]] was placed in the bridge phase on a sampled estimate $\lambda_2 \approx 0.0015$. the full-graph observation gave $\lambda_2 \approx 0.13$ ([[observing the spectral gap]], `tru/eval/spectral-gap-bostrom.md`), which by this table is maintenance: bostrom's structural work was already done, and its remaining value was semantic. the thresholds above are the original design's; they are untested against any second graph and are the first thing a run should calibrate.

## the mining loop

```
every epoch:
  read φ*, v₂, λ₂, J from tru
  read n, c(n) from the graph
  phase ← λ₂ against the table

  bridges:   candidates = top-K unlinked pairs by |v₂(i) − v₂(j)|,
             excluding same-cluster pairs; rank by Δλ₂ / c(n)
  mesh:      candidates = bypasses around articulation points;
             rank by component size × resilience gain / c(n)
  semantic:  candidates = hubs → undervalued files;
             rank by ΔJ / c(n)

  for each candidate: attach predicted settlement (rewards §6 marginal),
                      structural score, cost
  emit the proposal to the neuron
  record: link, phase, Δλ₂ predicted, cost at the time
  next epoch: compare realized Δλ₂ and settlement to the prediction;
              move the thresholds if they diverge
```

## convergence

with enough budget the graph reaches its phase transition: $\lambda_2$ above the critical value, $\phi^*$ no longer concentrated on a few hubs, connectivity past percolation. the exponential cost makes this slow and expensive; the Fiedler strategy makes it as cheap as the constraint allows, because well-chosen bridges need fewer links than random ones. the total cost from a starting graph is

$$C = \sum_{k=1}^{K} c_0 \, e^{\lambda (n_0 + k)}$$

with $K$ the number of bridges needed; minimizing $K$ is the whole point of reading $v_2$.

## the report

each epoch seer publishes, as a [[cyberlink]] into the graph it analyzes: current $\lambda_2$, $\Delta\lambda_2$ this epoch and its trend; the phase; the top proposals with predicted $\Delta\lambda_2$ and settlement; cost efficiency in bits of syntropy per token; the estimated links to phase transition on the current cost curve.

## what this method cannot see

only structure. it does not read content, so it cannot tell a true link from a plausible one; that is the market's job ([[market inhibition]]) and the neuron's. and its phases end: past maintenance, pure structural links stop clearing the settlement bar, and only focus-redistributing links remain worth mining until creation or decay reopens a region ([README](../README.md), when mining stops). the [learned policy](../roadmap/learned-policy.md) samples beyond the spectral signal, with this method as its teacher.

see [[observing the spectral gap]] for how $\lambda_2$ is read off the running iteration · [[foculus]] for what $\lambda_2$ means for finality · [[universal law]] for the cost curve
