# NFT Staking (Metaplex Core)

Anchor program for staking Metaplex Core NFTs from a collection, earning SPL rewards over time, and tracking how many NFTs are staked via collection-level Attributes.

## Architecture

```
Admin                          User
  |                              |
  | initialize                   | stake / claim_rewards / unstake
  v                              v
Config PDA ── authority ──> Rewards Mint (SPL, 6 decimals)
  ^                              |
  |                              v
Authority PDA <── Collection (Attributes: staked_count)
  ^                              |
  |                              v
  └── Asset (Attributes: staked, staked_at, last_claim_at)
       + FreezeDelegate while staked
```

### PDAs

| Account | Seeds |
|---------|-------|
| Update authority | `["update_authority", collection]` |
| Config | `["config", collection]` |
| Rewards mint | `["rewards_mint", collection]` |

### Instructions

1. **initialize** — Admin sets `rewards_bps` and `freeze_period`, creates config + rewards mint, adds collection Attributes plugin (`staked_count = 0`), transfers collection update authority to the program PDA.
2. **stake** — Freezes the NFT, sets asset attributes, increments collection `staked_count`.
3. **claim_rewards** — Mints SPL rewards based on time since `last_claim_at`. NFT stays staked and frozen.
4. **unstake** — Requires freeze period to elapse, clears staking attributes, thaws NFT, decrements `staked_count`.

### Reward formula

```
rewards = elapsed_seconds * rewards_bps / 10_000
```

Rewards are minted in the rewards mint's base units (6 decimals).

## Prerequisites

- [Rust](https://rustup.rs/)
- [Anchor 0.31+ / 1.0](https://www.anchor-lang.com/)
- [Surfpool](https://github.com/txpipe/surfpool) (optional, for `anchor test` Surfpool path)

## Build

```bash
cd builders/week-05/anchor-nft-staking
anchor build
```

Program ID: `9FAFuiroTiq89GaxGpYTVDQq1AbD3whjLpvWB43uhGGZ`

## Test

LiteSVM integration tests (recommended):

```bash
cargo test -p nft-staking --test staking --test staking_failures
```

Or via Anchor:

```bash
anchor test
```

### Test coverage

| Test | Description |
|------|-------------|
| `test_initialize` | Config, rewards mint, authority transfer, `staked_count = 0` |
| `test_stake` | Asset staked, count incremented |
| `test_claim_rewards_without_unstake` | Rewards minted, NFT still staked |
| `test_claim_then_unstake` | Claim then unstake in sequence |
| `test_unstake` | Unstake after freeze period |
| `test_multiple_stakes_update_count` | Collection count tracks multiple NFTs |
| Failure tests | Uninitialized stake, double stake, claim/unstake without stake, no rewards, freeze period |

## Project layout

```
programs/nft-staking/
├── src/
│   ├── lib.rs
│   ├── state.rs
│   ├── constants.rs
│   ├── error.rs
│   ├── instructions/
│   │   ├── initialize.rs
│   │   ├── stake.rs
│   │   ├── claim_rewards.rs
│   │   └── unstake.rs
│   └── utils/
├── tests/
│   ├── common/mod.rs
│   ├── fixtures/mpl_core.so
│   ├── staking.rs
│   └── staking_failures.rs
```

## Design notes

- **Separate claim and unstake** — Users accrue rewards while staked; `claim_rewards` updates `last_claim_at` without thawing the NFT. Unstaking is a distinct instruction gated by `freeze_period`.
- **Collection `staked_count`** — Stored on the collection Attributes plugin so anyone can read how many NFTs are staked in that collection.
- **Metaplex Core** — Uses Attributes + FreezeDelegate plugins per the [Metaplex staking guide](https://developers.metaplex.com/core/guides/staking).

## License

MIT
