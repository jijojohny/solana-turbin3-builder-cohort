# Instruction Introspection AMM

Anchor AMM for **Week 06 — Instruction Introspection**. Liquidity withdrawal is split into two instructions in the same transaction:

1. **`burn_lp`** — burns LP tokens from the depositor
2. **`withdraw_payout`** — reads the **immediately preceding** instruction via the Instructions sysvar, verifies it was `burn_lp` with matching accounts/amount, then pays out token A and B

This implements **Challenge 2**: an AMM that uses instruction introspection to verify LP burn before token payout, with burn and payout as separate instructions.

## Architecture

```
Depositor
    |
    |  initialize / deposit / swap
    v
Pool PDA ── owns ──> Vault A, Vault B, LP Mint
    ^
    |
    |  withdraw flow (single transaction, two instructions):
    |    1. burn_lp(lp_amount)
    |    2. withdraw_payout(min_a, min_b)  -- introspects prev ix
    v
Depositor receives token A + B
```

### PDAs

| Account | Seeds |
|---------|-------|
| Pool | `["pool", mint_a, mint_b]` |
| LP mint | `["lp_mint", pool]` |

`mint_a` must be lexicographically less than `mint_b`.

### Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize` | Creates pool, LP mint, and vault ATAs |
| `deposit` | Adds liquidity; mints LP tokens (constant-product, initial LP = √(a×b)) |
| `burn_lp` | Burns LP from depositor (standalone or as first ix in withdraw tx) |
| `withdraw_payout` | Introspects previous `burn_lp`, then transfers A/B from vaults |
| `swap_a_for_b` | Swap token A for B (0.30% default fee) |
| `swap_b_for_a` | Swap token B for A |

### Instruction introspection

`withdraw_payout` loads the Instructions sysvar and checks that the previous instruction in the transaction:

- Targets this program
- Has the `burn_lp` discriminator
- Uses the same depositor, pool, LP mint, and depositor LP ATA
- Specifies a non-zero `lp_amount`

Payout amounts are computed against the **pre-burn** LP supply (`current_supply + burned_amount`), since `burn_lp` runs first in the same transaction.

## Prerequisites

- [Rust](https://rustup.rs/) (1.89+)
- [Anchor 1.0](https://www.anchor-lang.com/)

## Build

```bash
cd builders/week-06/anchor-introspection-amm
anchor build
```

Program ID: `2QMKjmtkFRnDQJihz9DEnNBFGoUqEXqXc1kQBVjrvmyB`

## Test

LiteSVM integration tests cover every instruction plus failure paths (missing burn, wrong order, slippage):

```bash
anchor test
# or
cargo test -p introspection-amm --test amm --test amm_failures
```

**12 tests passing** (6 happy path + 6 failure cases).

### Example withdraw transaction

Clients must send **both** instructions in one transaction, in order:

```rust
let ixs = [
    burn_lp_ix(depositor, mint_a, mint_b, lp_amount),
    withdraw_payout_ix(depositor, mint_a, mint_b, min_a, min_b),
];
// send_multi(ixs)
```

## Project layout

```
programs/introspection-amm/
├── src/
│   ├── instructions/
│   │   ├── initialize.rs
│   │   ├── deposit.rs
│   │   ├── burn_lp.rs
│   │   ├── withdraw_payout.rs
│   │   └── swap.rs
│   ├── utils/
│   │   └── introspection.rs   # burn_lp verification via Instructions sysvar
│   ├── math.rs
│   └── state.rs
└── tests/
    ├── amm.rs
    └── amm_failures.rs
```

## Week 06 challenge mapping

| Requirement | Status |
|-------------|--------|
| Challenge 2: AMM with introspection | Done |
| Burn and payout as separate instructions | `burn_lp` + `withdraw_payout` |
| Written from scratch | Yes |
| Tests for all instructions | 12 LiteSVM tests |
| README | This file |
