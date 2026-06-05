# anchor-amm

Constant-product AMM (initialize, deposit, withdraw, swap) with LiteSVM integration tests.

## Prerequisites

- [Anchor](https://www.anchor-lang.com/) 1.0+
- [Surfpool](https://github.com/solana-foundation/surfpool) 1.1.2+ (required for `anchor test` / `anchor localnet`)

Install Surfpool:

```bash
curl -sL https://run.surfpool.run/ | bash
surfpool --version
```

Ensure `~/.local/bin` is on your `PATH` (the installer places `surfpool` there).

## Test

```bash
anchor build
anchor test
```

Or run LiteSVM tests only:

```bash
cargo test -p anchor-amm
```
