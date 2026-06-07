# NFT Marketplace (Metaplex Core)

Anchor program for listing Metaplex Core NFTs, buying with SOL or SPL tokens, and negotiating via escrowed offers.

## Architecture

```
Admin --initialize--> Marketplace PDA (fee_bps, treasury)
Maker --list--> Listing PDA + TransferDelegate(listing)
Buyer --buy / buy_with_token--> pay maker + treasury fee, NFT transferred
Buyer --make_offer--> Offer PDA (SOL escrow)
Maker --accept_offer--> pay from escrow, NFT transferred
Buyer --cancel_offer--> refund escrow, close Offer
Maker --delist--> revoke delegate, close Listing
```

### PDAs

| Account | Seeds |
|---------|-------|
| Marketplace | `["marketplace"]` |
| Listing | `["listing", asset]` |
| Offer | `["offer", asset, buyer]` |

### Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize` | Create marketplace config with treasury and fee |
| `list` | List NFT at price in SOL (`NATIVE_MINT`) or SPL (`payment_mint`) |
| `delist` | Maker removes listing and transfer delegate |
| `buy` | Pay listed SOL price; split to maker + treasury |
| `buy_with_token` | Pay listed SPL price via `transfer_checked` to maker/treasury ATAs |
| `make_offer` | Escrow SOL in Offer PDA |
| `accept_offer` | Maker accepts offer amount instead of list price |
| `cancel_offer` | Buyer refunds escrow and closes offer |

### Payment split

```
fee = price * fee_bps / 10_000
maker_amount = price - fee
```

## Prerequisites

- Rust, Anchor 1.0+, Solana CLI

## Build

```bash
cd builders/week-05/anchor-nft-marketplace
anchor build
```

Program ID: `9XKHUKki3mBjhkFrwvFqwdenTCykw8FH3YYdqT7f2Uru`

## Test

```bash
cargo test -p nft-marketplace --test marketplace --test marketplace_failures
```

Or:

```bash
anchor test
```

### Test coverage

**Happy path:** initialize, list/delist, buy (SOL), buy_with_token (SPL), make/cancel offer, accept offer, accept below list price

**Failures:** buy own listing, unauthorized delist, wrong payment type, offer on own asset, unauthorized cancel, double list

## Design notes

- **Escrowless listings** use Metaplex Core `TransferDelegate` with the Listing PDA as authority; `buy` signs transfer via listing seeds.
- **SPL payments** use `Interface<TokenInterface>` and `transfer_checked` (Token-2022 ready). Treasury receives fees via ATA, not a raw system account.
- **Offers** escrow SOL in the Offer PDA; `accept_offer` distributes via PDA-signed system transfers.

## License

MIT
