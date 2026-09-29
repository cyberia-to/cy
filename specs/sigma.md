---
title: sigma — the sum
tags: sigma, cy, spec, core
alias: sigma spec, token registry, three measurements
crystal-type: spec
crystal-domain: cyber
status: draft
date: 2026-09-29
---
# sigma — the sum

## the one thing

sigma shows the total balance of a neuron, valued in a token. everything else on this page exists to make that number true and to let the owner decide what "true" means.

## the registry

sigma keeps the owner's registry of tokens. a token is any denomination the [[cybergraph]] can carry in a [[cyberlink]]'s box, fungible or not: a coin of [[tok]] (TSP-1), a card (TSP-2), a foreign asset the owner chooses to track. an entry is

```
token      the particle that names it
kind       coin | card
unit       the smallest count, and the scale it is shown in
price      a method
balance    a method
supply     a method
```

the registry is the owner's, kept in the book of the neuron: adding a token is a cyberlink from the neuron's name to the token's particle under the sigma dialect, and so is changing a method. two neurons may value the same token by different rules and both are right for their own sum.

## the three measurements

every measurement is taken **of a token, on a neuron, at a moment**, and each token answers through its own method.

| measurement | question | signature |
|---|---|---|
| price | what is one unit of this token worth in that token | `price(token, in: token, at: moment) → amount` |
| balance | how much of this token does this neuron hold | `balance(token, neuron, at: moment) → amount` |
| supply | how much of this token exists | `supply(token, at: moment) → amount` |

a method is a rule that produces the number, and its provenance travels with the number: sigma reports the value and the tier it came from (proof-verified, anchor-verified, unproven, typed by hand), never the value alone. the tiers are those of [[cy/specs/state|state]].

## methods

a token may use any method for any measurement. the standard set ships with sigma; the owner may write others as [[rune]] cells.

price
- `book` — the price the neuron's own book records for the token (the last box it moved, or a price link the neuron wrote)
- `market` — a quote from a named market or oracle, with the quote's identity and age
- `formula` — a rule over other measurements: a share of a supply, a basket, a bonding curve, a fixed ratio to another token
- `hand` — a number the owner typed; the honest default for a token nobody quotes

balance
- `book` — the sum of the neuron's boxes in the token, read from the [[cybergraph]] (the only balance that is provable)
- `chain` — a balance answered by a network the token lives on, through [[cy/specs/state|state]]
- `hand` — for what is held off any book

supply
- `conservation` — mints minus burns per [[tok]]'s conservation law; provable for tok coins
- `chain` — the supply a network reports
- `fixed` — a cap or a declared number
- `hand`

turnover is not a measurement. what moved, and when, is the [[cy/specs/log|log]]'s business; sigma reads it when a method needs a rate and ignores it otherwise.

## the derived quantities

with three measurements every token yields the rest without a fourth method:

- **value** of a holding: `balance(t, n) × price(t, in: k)` for any numeraire k
- **the sum**: `Σ_t balance(t, n) × price(t, in: k)` over the registry — the number sigma exists for
- **share**: `balance(t, n) / supply(t)` — how much of a token this neuron is
- **capitalisation**: `supply(t) × price(t, in: k)`
- **the price of a token in any other**: `price(a, in: b) = price(a, in: k) / price(b, in: k)` for any k both are priced in; a token priced only by `hand` prices everything else through that hand

the numeraire is the owner's choice and may be any token in the registry, including one whose price is a formula. the sum in [[cx|cx]] and the sum in a coin of the neuron's own book are the same computation with a different k.

## what sigma is not

- not a wallet: it holds nothing and signs nothing; [[cy/specs/vault|vault]] holds and signs, sigma spends by asking it
- not a market: it never quotes, it reads quotes
- not an accountant of flows: turnover, yield and cost basis are derived from the log by whoever needs them

## surfaces

one registry, three faces: `cy sigma` prints the sum and the table in a terminal; the sigma world in [[cyb]] paints the same table; a cell may ask any measurement by name. the number on the page is last known and the method runs behind it ([[cyb/parts/live|live]]).

## open

- the sigma dialect of [[neural]]: the exact sentences for "register token", "set method", "price in"
- which methods are provable: `book` and `conservation` today; `market` never; `formula` when the inputs are
- the seed: the owner's terminal sigma (positions, nine price sources, a cache) is the first implementation of `market` and `hand` methods and is not public
