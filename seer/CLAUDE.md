# seer — agent instructions

seer is a miner. it mines the mint of any chain that pays for focus — every token has its own home book (cyber/research/oikos.md: one token, one chain), $CYB's is the reference — by finding the cyberlinks that shift focus most per unit of will and handing them to the neuron to sign. read README.md first. four invariants are the whole boundary:

1. nothing in this repo computes φ* or touches tru's focusing iteration. learning lives outside the contraction.
2. the objective is the neuron's expected settlement under rewards.md, net of cost. structural signals (Δλ₂, articulation points) are priors, never the reward.
3. seer never creates a file. it proposes edges between files that exist. creation is another component's job.
4. every proposal carries a predicted settlement, and every settled epoch scores it against tru's measurement. report truthfully — the warriors doctrine.

## layout

- `specs/` — contracts (interface, densification). source of truth; when code and spec disagree, fix the spec first, then propagate.
- `docs/` — why seer exists and where mining ends. explains, never restates the specs.
- `roadmap/` — proposals with `status:` frontmatter (proposal → accepted → in spec). rejected proposals stay for rationale.

## conventions

- pages are markdown with YAML frontmatter; wiki-links `[[page]]` resolve across the cyber graph. never write `[[term]]s` — put plurals in `alias:`.
- no bold in graph pages. state what a thing is; never define by negation.
- commit by default, atomic, conventional prefixes (`feat:`, `fix:`, `docs:`, `chore:`).
- fixed-point over the Goldilocks field wherever a number must agree across machines (tru/specs/arithmetic.md). private ranking may use floats; anything published as a claim may not.

## companion repos

| repo | seer reads | seer writes |
|---|---|---|
| tru | φ*, cyberank, the propose-phase marginal Δφ⁺, λ₂, spectral positions, syntropy | nothing |
| cybergraph | signals, cyberlinks, particles | nothing directly — mined links go to the neuron |
| cyb | the neuron's keys, will, context | candidate signals for the owner to sign |
| warriors | the mining and interface doctrines | nothing |
