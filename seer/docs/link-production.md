---
tags: cyber, cyb, seer, article
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-03-24
alias: link production, research/link production, the intelligence problem, link production problem
---
# link production: the intelligence problem

## the gap

the protocol stack validates, orders, and propagates links: [[zheng]] proves a link valid, the hash chain orders it, [[bbg]] commits it as polynomial state and [[DAS]] proves it available, [[foculus]] merges, [[focus]] economics rate-limit production, [[temporal decay]] prunes. none of it decides what to link. production is the intelligence problem — the thing the protocol exists to serve. without it the [[cybergraph]] is an empty authenticated structure. [[seer]] is the component that owns this problem on the neuron's side.

## what production is

a [[cyberlink]] is $(p, q, \tau, a, v)$ — source and target [[particles]], token, amount, [[valence]]; the signing [[neuron]] and block belong to the containing [[signal]]. producing one means a neuron decides $p$ is relevant to $q$ and spends [[focus]] to say so. that decomposes:

```
1. discovery     what files exist, or should?
2. evaluation    which connections would improve the graph?
3. decision      is the improvement worth the cost?
4. commitment    sign the cyberlink, spend, prove
5. propagation   sync through the protocol
```

4 and 5 are the protocol. 3 is economics — will, conviction, the exponential cost of [[universal law]]. 1 and 2 are intelligence. seer takes the half of them that is mining: connecting files that exist. discovering or making files that do not is creation, another component's job.

## what exists

| tool | does | step | limit |
|---|---|---|---|
| [densification](../specs/densification.md) | mining, analytical: Fiedler vector → bridge and bypass candidates | discovery + evaluation, existing→existing | structure only, no content |
| [learned policy](../roadmap/learned-policy.md) | mining, learned: sample links in proportion to expected settlement | discovery + evaluation, existing→existing | proposal, unbuilt |
| human neurons | judgment: read, think, link | all | slow, does not scale |
| LLM agents | synthesis: generate content, propose links | discovery + creation of new files | hallucination, no stake in the outcome |
| [[tru]] | the measure: $\phi^*$, $\Delta\phi^+$, settlement | evaluation, exact | measures; never proposes |
| [[temporal decay]] | pruning: low-energy links leave | negative evaluation, retroactive | removes; never proposes |

the gap: nothing yet connects content understanding to link decision at scale with economic accountability. densification understands structure and not content; agents understand content and do not pay for mistakes; humans understand both and do not scale.

## existing→existing and existing→new

both files known — the link asserts a relationship. information added: the edge, $O(1)$. this is search in an $O(N^2)$ space: densification navigates it analytically, a learned method by sampling. it is retrieval — recognizing a pattern in what is already known.

target file new — the link asserts a relationship to content that did not exist in the graph. information added: the content plus the edge. this is synthesis in an unbounded space. it is generation — writing, research, observation.

at the protocol level the two are one thing: both are cyberlinks, both cost, both carry proofs, [[bbg]] does not distinguish them. at the information level they differ — mutual information $I(p;q)$ against entropy $H(\text{new})$. at the economic level they differ — the second pays storage and carries the risk that the content is worthless or redundant. and they are a spectrum:

```
pure connection      Cat → Animal                       both known, obvious
informed connection  Cat → an obscure paper on cats     known, hard to find
partial creation     Cat → a curated summary            derivative, adds structure
full creation        Cat → original research            new knowledge
pure creation        new topic → new content            both new
```

seer mines the left end — pure and informed connection, where every candidate can be priced exactly before commitment. agents work across the middle; original researchers at the right. the line between mining and creation is the line between a value that can be computed from public state and one that exists only after the file is made.

## what is missing

1. content discovery at scale — a neuron sees its neighbourhood; the graph has billions of particles. personalized focus from the neuron's own links as seeds is the natural recommender: high personalized $\phi^*$, not yet linked, is a candidate.
2. novelty detection — particle identity is $H(\text{content})$, so paraphrases accumulate. focus economics punish copies after the fact ([[temporal decay]]); the storage was already paid. a pre-commitment similarity check is missing.
3. quality before commitment — focus is irreversible. [[rewards]] §6 gives the neuron its exact standalone marginal $\Delta\phi^+_\nu$ for a candidate set, locality-bounded and provable; what is missing is a surrogate cheap enough to price thousands of candidates per step. that is the first open question of the [learned policy](../roadmap/learned-policy.md).
4. cold start — an empty graph has no $\phi^*$ to improve. the sequence is finite: seed files, free early links, densification on the seed graph, and once $\lambda_2$ clears its threshold the tri-kernel's $\phi^*$ becomes meaningful and economics take over.
5. the agent-to-link interface — an agent needs to discover what the graph lacks, generate what fills it, score its own output, and sign. seer's [interface](../specs/interface.md) is the contract for the first three; the neuron's key ([[cyb]]) is the fourth. a proposal produced by a [[nox]] program carries a [[zheng]] trace, so the decision process itself is provable.

## the production stack

```
0  content        humans, agents, imports        → raw content
1  files          hemera hash, bbg + DAS         → files, each named by its particle
2  discovery      densification · personalized   → candidate links
                  focus · learned proposal ·
                  agent reasoning
3  evaluation     Δλ₂ (structure) · Δφ⁺ (the     → scored candidates
                  measure, from tru) · surprise ρ
                  (novelty) · cost
4  decision       expected settlement net of      → committed links
                  cost, under the neuron's budget
5  protocol       zheng · ordering · bbg · DAS ·  → verified graph state
                  foculus
```

layers 0–4 are the intelligence problem and seer's territory; layer 5 is the protocol, whose readiness is tracked at [[soft3/status]].

## the deep point

existing→existing is intelligence: seeing what connects things that are already known. existing→new is knowledge: the graph knows more than before. the cybergraph needs both — intelligence without new knowledge is a perfect static map; knowledge without intelligence is a dump. the balance follows the same curve as densification's phases: sparse graphs favour connection, dense graphs favour creation, and $\lambda_2$ says where the graph stands. no planner sets the ratio; the exponential cost and the spectral return do.

## open questions

1. a surrogate for $\Delta\phi^+$ cheap enough for real-time evaluation of many candidates — personalized push-back on the ego-net may give $O(1/\varepsilon)$
2. recovering storage from decayed files, so low-quality creation is not permanently paid for
3. the agent architecture that spans the spectrum — structure from densification, content from a model, sampling from a learned proposer, accountability from settlement — and whether one architecture covers it
4. whether production has a fixed point: agents optimize $\Delta\phi^+$, $\phi^*$ moves in response. equilibrium or oscillation? see [[observing the spectral gap]] for the convergence side
5. an information-theoretic ceiling on how complete the graph can get under finite collective focus ([[knowledge completeness]])

see [[seer]] · [[tru]] for the measure · [[cybergraph model architecture|the graph as generative model]] · [[collective focus theorem]] · [[universal law]] · [[tri-kernel architecture]]
