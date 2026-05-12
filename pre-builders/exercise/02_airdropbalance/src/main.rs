use solana_client::rpc_client::RpcClient;
use solana_address::Address;
use std::env;
use std::str::FromStr;

const RPC_URL: &str = "https://api.devnet.solana.com";
const ADDRESS: &str = "EibRsRoMiPD7yndP7YJbZt5Ut19poNqsjs3BvvTQ5rgp";
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;


pub fn get_balance(client: &RpcClient,address: &Address) -> u64 {
    let balance = client.get_balance(address);
    balance.unwrap_or(0) as u64
}

fn main() {
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| RPC_URL.to_string());
    let address_str = env::var("ADDRESS").unwrap_or_else(|_| ADDRESS.to_string());
    let client = RpcClient::new(rpc_url);
    let address = Address::from_str(&address_str).expect("invalid address");
    let balance = get_balance(&client, &address);
    let whole_sol = balance / LAMPORTS_PER_SOL;
    let fractional_lamports = balance % LAMPORTS_PER_SOL;

    println!("Balance (lamports): {}", balance);
    println!("Balance (SOL): {}.{:09}", whole_sol, fractional_lamports);
}