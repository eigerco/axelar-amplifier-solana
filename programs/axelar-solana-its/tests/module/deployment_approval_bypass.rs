use axelar_solana_gateway_test_fixtures::base::FindLog;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::instructions::CreateV1Builder;
use mpl_token_metadata::types::TokenStandard;
use solana_program_test::tokio;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer as _;
use test_context::test_context;

use axelar_solana_its::state::token_manager::Type as TokenManagerType;
use evm_contracts_test_suite::evm_contracts_rs::contracts::custom_test_token::CustomTestToken;
use evm_contracts_test_suite::ContractMiddleware;
use interchain_token_transfer_gmp::GMPPayload;

use crate::{fetch_first_call_contract_event_from_tx, ItsTestContext};

/// Helper function to set up a custom token with a specific deployer
async fn setup_custom_token_with_deployer(
    ctx: &mut ItsTestContext,
    deployer_keypair: &Keypair,
    token_manager_type: TokenManagerType,
    token_name: &str,
    token_symbol: &str,
    salt_seed: &[u8],
) -> anyhow::Result<([u8; 32], CustomTestToken<ContractMiddleware>, Pubkey)> {
    let deployer = deployer_keypair.pubkey();
    let salt = solana_sdk::keccak::hash(salt_seed).to_bytes();
    let custom_token = ctx
        .evm_signer
        .deploy_axelar_custom_test_token(token_name.to_owned(), token_symbol.to_owned(), 18)
        .await?;

    let custom_solana_token = ctx
        .solana_chain
        .fixture
        .init_new_mint(deployer, spl_token_2022::id(), 9)
        .await;

    let (metadata_pda, _) = Metadata::find_pda(&custom_solana_token);
    let metadata_ix = CreateV1Builder::new()
        .metadata(metadata_pda)
        .token_standard(TokenStandard::Fungible)
        .mint(custom_solana_token, false)
        .authority(deployer)
        .update_authority(deployer, true)
        .payer(ctx.solana_wallet)
        .is_mutable(false)
        .name(token_name.to_owned())
        .symbol(token_symbol.to_owned())
        .uri(String::new())
        .seller_fee_basis_points(0)
        .instruction();

    let register_metadata = axelar_solana_its::instruction::register_token_metadata(
        deployer,
        custom_solana_token,
        spl_token_2022::id(),
        0,
    )?;

    let tx = ctx
        .solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[metadata_ix, register_metadata],
            &[
                deployer_keypair.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;
    let call_contract_event = fetch_first_call_contract_event_from_tx(&tx.unwrap());

    let GMPPayload::RegisterTokenMetadata(register_message) =
        GMPPayload::decode(&call_contract_event.payload)?
    else {
        panic!("wrong message");
    };

    assert_eq!(
        register_message.token_address.as_ref(),
        custom_solana_token.as_ref()
    );

    ctx.evm_its_contracts
        .interchain_token_service
        .register_token_metadata(custom_token.address(), 0.into())
        .send()
        .await?
        .await?;

    let token_id = axelar_solana_its::linked_token_id(&deployer, &salt);
    let register_custom_token_ix = axelar_solana_its::instruction::register_custom_token(
        deployer,
        salt,
        custom_solana_token,
        token_manager_type,
        spl_token_2022::id(),
        None,
    )?;

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[register_custom_token_ix],
            &[
                deployer_keypair.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    let link_token_ix = axelar_solana_its::instruction::link_token(
        deployer,
        salt,
        ctx.evm_chain_name.clone(),
        custom_token.address().as_bytes().to_vec(),
        token_manager_type,
        vec![],
        0,
    )?;

    let tx = ctx
        .solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[link_token_ix],
            &[
                deployer_keypair.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;
    let call_contract_event = fetch_first_call_contract_event_from_tx(&tx.unwrap());
    ctx.relay_to_evm(&call_contract_event.payload).await;

    Ok((token_id, custom_token, custom_solana_token))
}

/// Test that demonstrates the deployment approval bypass vulnerability
/// where an outdated minter can use a different token manager for authorization
#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_deployment_approval_bypass_with_fake_token_manager(
    ctx: &mut ItsTestContext,
) -> anyhow::Result<()> {
    // Create Alice as the original deployer/minter
    let alice = Keypair::new();
    ctx.solana_chain
        .fixture
        .fund_account(&alice.pubkey(), 10_000_000_000)
        .await;

    // Step 1: Alice creates TokenA (native interchain token)
    let salt_a = solana_sdk::keccak::hash(b"token-a-salt").to_bytes();
    let deploy_token_a_ix = axelar_solana_its::instruction::deploy_interchain_token(
        alice.pubkey(),
        salt_a,
        "Token A".to_owned(),
        "TA".to_owned(),
        9,
        1000,
        Some(alice.pubkey()),
    )?;

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[deploy_token_a_ix],
            &[
                alice.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    let _token_id_a = axelar_solana_its::interchain_token_id(&alice.pubkey(), &salt_a);
    
    // Step 2: Alice creates a deployment approval for TokenA
    let destination_chain = "polygon".to_string();
    let destination_minter = b"0x1234567890123456789012345678901234567890".to_vec();

    let approve_deployment_ix =
        axelar_solana_its::instruction::approve_deploy_remote_interchain_token(
            alice.pubkey(),
            alice.pubkey(), // Alice is the deployer
            salt_a,
            destination_chain.clone(),
            destination_minter.clone(),
        )?;

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[approve_deployment_ix],
            &[
                alice.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    // Step 3: Alice creates TokenB (native interchain token)  
    let salt_b = solana_sdk::keccak::hash(b"token-b-salt").to_bytes();
    let deploy_token_b_ix = axelar_solana_its::instruction::deploy_interchain_token(
        alice.pubkey(),
        salt_b,
        "Token B".to_owned(),
        "TB".to_owned(),
        9,
        1000,
        Some(alice.pubkey()),
    )?;

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[deploy_token_b_ix],
            &[
                alice.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    let token_id_b = axelar_solana_its::interchain_token_id(&alice.pubkey(), &salt_b);

    // Step 4: Alice attempts to deploy TokenA remotely using TokenB's token manager for authorization
    // This should fail after the fix
    
    let mut deploy_remote_ix = axelar_solana_its::instruction::deploy_remote_interchain_token_with_minter(
        alice.pubkey(),
        salt_a,
        alice.pubkey(), // Alice as the deployer
        destination_chain.clone(),
        destination_minter,
        0, // no gas
    )?;

    // The vulnerability: Replace TokenA's token_manager_pda with TokenB's token_manager_pda
    // This allows Alice to use her current minter privileges on TokenB to deploy TokenA
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let token_b_manager_pda = axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id_b).0;
    
    // Find the token_manager_account position in the instruction accounts (should be account index 3)
    deploy_remote_ix.accounts[3].pubkey = token_b_manager_pda;

    let result = ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[deploy_remote_ix],
            &[
                alice.insecure_clone(),
                ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    // After the fix, this attack should fail
    assert!(
        result.is_err(),
        "Expected transaction to fail due to token manager/mint mismatch"
    );

    let error_tx = result.unwrap_err();
    assert!(
        error_tx.find_log("Provided seeds do not result in a valid address").is_some(),
        "Expected token manager validation error message"
    );

    Ok(())
}