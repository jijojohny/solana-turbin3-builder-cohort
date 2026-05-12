use solana_address::Address;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub fn new_wallet() -> Keypair {
    Keypair::new()
}
pub fn wallet_address(wallet: &Keypair) -> Address {
    wallet.pubkey()
}

fn main() {
    let wallet = new_wallet();
    let address = wallet_address(&wallet);
    println!("Wallet address: {}", address)
}