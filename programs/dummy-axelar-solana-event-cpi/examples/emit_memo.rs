use anyhow::Result;
use dummy_axelar_solana_event_cpi::instruction::emit_event;
use solana_cli_config::{Config, CONFIG_FILE};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::AccountMeta,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    transaction::Transaction,
};
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <memo_message>", args[0]);
        std::process::exit(1);
    }

    let memo = args[1].clone();

    // Load Solana CLI config
    let config_file = CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Unable to determine config file path"))?;

    let cli_config = if Path::new(config_file).exists() {
        Config::load(config_file)?
    } else {
        Config::default()
    };

    // Use the RPC URL from config, fallback to devnet
    let rpc_url = cli_config.json_rpc_url;
    println!("Using RPC URL: {}", rpc_url);

    let client = RpcClient::new_with_commitment(&rpc_url, CommitmentConfig::confirmed());

    // Use keypair from config
    let payer = Keypair::read_from_file(cli_config.keypair_path)
        .map_err(|_| anyhow::anyhow!("Cannot load keypair"))?;

    println!("Payer: {}", payer.pubkey());
    println!("Memo: {}", memo);

    // Create the basic instruction
    let mut instruction = emit_event(&payer.pubkey(), memo)?;

    // Add required accounts for event CPI functionality
    use dummy_axelar_solana_event_cpi::ID as PROGRAM_ID;

    // Derive the event authority PDA
    let (event_authority, _bump) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[event_cpi::EVENT_AUTHORITY_SEED],
        &PROGRAM_ID,
    );

    // Add the event authority account
    instruction
        .accounts
        .push(AccountMeta::new_readonly(event_authority, false));

    // Add the program account
    instruction
        .accounts
        .push(AccountMeta::new_readonly(PROGRAM_ID, false));

    println!("Event Authority: {}", event_authority);

    // Create and send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("Transaction signature: {}", signature);

    Ok(())
}
