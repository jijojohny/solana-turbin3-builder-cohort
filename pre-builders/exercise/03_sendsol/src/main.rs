use solana_address::Address;
use solana_signer::Signer;
use solana_client::rpc_client::RpcClient;
use sdt::env;
use solana_transaction::Transaction;
use solana_transaction::transfer::TransferInstruction;

fn send_sol(client: &RpcClient, from_address: &Address, to_address: &Address, amount: u64) -> Result<(), Box<dyn Error>> {
    let transaction = Transaction::new_signed_with_payer(
        &[TransferInstruction::new(from_address, to_address, amount)],
        Some(&from_address.signer()),
        &client,
        &[&from_address.signer().clone()],
    )?;
    let signature = client.send_transaction(&transaction).await?;
    println!("Transaction sent: {}", signature);
    Ok(())
}

#[tokio::main]
async fn main() {
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| RPC_URL.to_string());
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentLevel::Processed);
    let from_address = Address::from_str(&env::var("FROM_ADDRESS").unwrap_or_else(|_| FROM_ADDRESS.to_string())).expect("invalid from address ");
    let to_address = Address::from_str(&env::var("TO_ADDRESS").unwrap_or_else(|_| TO_ADDRESS.to_string())).expect("invalid to address");
    let amount = env::var("AMOUNT").unwrap_or_else(|_| AMOUNT.to_string()).parse::<u64>().expect("invalid amount");


    send_sol(&client, &from_address, &to_address, amount).await.expect("Failed to send transaction");
    println!("Transaction sent successfully");
}