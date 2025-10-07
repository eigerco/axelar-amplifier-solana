use axelar_solana_gateway_test_fixtures::{assert_msg_present_in_logs, base::TestFixture};
use event_cpi_test_utils::assert_event_cpi;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas(#[case] token_program_id: Pubkey) {
    // Setup the test fixture and deploy the gas service program

    use axelar_solana_gas_service::events::SplGasAddedEvent;

    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup a mint and mint some tokens to the payer
    let payer = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;
    let mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;
    let payer_ata = test_fixture
        .init_associated_token_account(&mint, &payer.pubkey(), &token_program_id)
        .await;
    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &mint,
            &payer_ata,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Setup the config_pda ATA
    let config_pda_ata = test_fixture
        .init_associated_token_account(&mint, &gas_utils.config_pda, &token_program_id)
        .await;

    // Fetch payer and config_pda ATA balances before
    let payer_token_before = test_fixture.get_token_account(&payer_ata).await.amount;
    let config_pda_token_before = test_fixture.get_token_account(&config_pda_ata).await.amount;

    // Prepare args
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    // Create the instruction for paying gas fees with SPL tokens
    let ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),
        &payer_ata,
        &mint,
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // First simulate to check events
    let simulation_result = test_fixture
        .simulate_tx_with_custom_signers(
            &[ix.clone()],
            &[
                // pays for transaction fees
                &test_fixture.payer.insecure_clone(),
                // payer signs to transfer tokens
                &payer,
            ],
        )
        .await
        .unwrap();

    // Assert event emitted
    let inner_ixs = simulation_result
        .simulation_details
        .unwrap()
        .inner_instructions
        .unwrap()
        .first()
        .cloned()
        .unwrap();
    assert!(!inner_ixs.is_empty());

    let expected_event = SplGasAddedEvent {
        config_pda: gas_utils.config_pda,
        config_pda_ata,
        mint,
        token_program_id,
        tx_hash,
        ix_index,
        event_ix_index,
        refund_address,
        gas_fee_amount: gas_amount,
    };

    assert_event_cpi(&expected_event, &inner_ixs);

    // Execute the transaction
    let _res = test_fixture
        .send_tx_with_custom_signers(
            &[ix],
            &[
                // pays for transaction fees
                &test_fixture.payer.insecure_clone(),
                // payer signs to transfer tokens
                &payer,
            ],
        )
        .await
        .unwrap();

    // Fetch payer and config_pda ATA balances before
    let payer_token_after = test_fixture.get_token_account(&payer_ata).await.amount;
    let config_pda_token_after = test_fixture.get_token_account(&config_pda_ata).await.amount;

    // Assert that tokens got transferred
    assert_eq!(payer_token_after, payer_token_before - gas_amount);
    assert_eq!(config_pda_token_after, config_pda_token_before + gas_amount);
}

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas_missing_signer(#[case] token_program_id: Pubkey) {
    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup a mint and mint some tokens to the payer
    let payer = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;
    let mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;
    let payer_ata = test_fixture
        .init_associated_token_account(&mint, &payer.pubkey(), &token_program_id)
        .await;
    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &mint,
            &payer_ata,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Setup the config_pda ATA
    let _config_pda_ata = test_fixture
        .init_associated_token_account(&mint, &gas_utils.config_pda, &token_program_id)
        .await;

    // Create the instruction without making the payer a signer
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    let mut ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),
        &payer_ata,
        &mint,
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // Remove the signer flag from the sender account
    ix.accounts[0].is_signer = false;

    // Send transaction (should fail)
    let res = test_fixture
        .send_tx_with_custom_signers(&[ix], &[&test_fixture.payer.insecure_clone()])
        .await;

    // Assert that the transaction failed with MissingRequiredSignature error
    assert!(res.is_err());
    assert_msg_present_in_logs(
        res.unwrap_err(),
        "missing required signature for instruction",
    );
}

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas_invalid_sender_ata_wrong_mint(#[case] token_program_id: Pubkey) {
    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup two different mints
    let payer = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;

    // Create the correct mint for the instruction
    let correct_mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create a different mint for the payer's ATA
    let wrong_mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create payer's ATA for the wrong mint
    let payer_ata_wrong_mint = test_fixture
        .init_associated_token_account(&wrong_mint, &payer.pubkey(), &token_program_id)
        .await;

    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &wrong_mint,
            &payer_ata_wrong_mint,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Setup the config_pda ATA for the correct mint
    let _config_pda_ata = test_fixture
        .init_associated_token_account(&correct_mint, &gas_utils.config_pda, &token_program_id)
        .await;

    // Create the instruction with mismatched mint/ATA
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    let ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),
        &payer_ata_wrong_mint, // Using ATA for wrong mint
        &correct_mint,         // But specifying the correct mint
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // Send transaction (should fail)
    let res = test_fixture
        .send_tx_with_custom_signers(&[ix], &[&test_fixture.payer.insecure_clone(), &payer])
        .await;

    // Assert that the transaction failed with InvalidAccountData error
    assert!(res.is_err());
    assert_msg_present_in_logs(res.unwrap_err(), "invalid account data for instruction");
}

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas_invalid_sender_ata_wrong_owner(#[case] token_program_id: Pubkey) {
    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup mint and two different owners
    let payer = Keypair::new();
    let wrong_owner = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;

    let mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create ATA for the wrong owner (not the payer)
    let wrong_owner_ata = test_fixture
        .init_associated_token_account(&mint, &wrong_owner.pubkey(), &token_program_id)
        .await;

    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &mint,
            &wrong_owner_ata,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Setup the config_pda ATA
    let _config_pda_ata = test_fixture
        .init_associated_token_account(&mint, &gas_utils.config_pda, &token_program_id)
        .await;

    // Create the instruction with wrong owner's ATA
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    let ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),  // Payer is the signer
        &wrong_owner_ata, // But using wrong owner's ATA
        &mint,
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // Send transaction (should fail)
    let res = test_fixture
        .send_tx_with_custom_signers(&[ix], &[&test_fixture.payer.insecure_clone(), &payer])
        .await;

    // Assert that the transaction failed with InvalidAccountData error
    assert!(res.is_err());
    assert_msg_present_in_logs(res.unwrap_err(), "invalid account data for instruction");
}

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas_invalid_config_pda_ata_wrong_mint(#[case] token_program_id: Pubkey) {
    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup two different mints
    let payer = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;

    // Create the correct mint for the payer
    let correct_mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create a different mint for config_pda's ATA
    let wrong_mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create payer's ATA for the correct mint
    let payer_ata = test_fixture
        .init_associated_token_account(&correct_mint, &payer.pubkey(), &token_program_id)
        .await;

    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &correct_mint,
            &payer_ata,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Setup the config_pda ATA for the wrong mint
    let config_pda_ata_wrong_mint = test_fixture
        .init_associated_token_account(&wrong_mint, &gas_utils.config_pda, &token_program_id)
        .await;

    // Create the instruction with mismatched config ATA
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    // Manually construct the instruction to use wrong config_pda_ata
    let mut ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),
        &payer_ata,
        &correct_mint,
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // Replace the config_pda_ata account with the wrong one
    ix.accounts[3].pubkey = config_pda_ata_wrong_mint;

    // Send transaction (should fail)
    let res = test_fixture
        .send_tx_with_custom_signers(&[ix], &[&test_fixture.payer.insecure_clone(), &payer])
        .await;

    // Assert that the transaction failed with InvalidAccountData error
    assert!(res.is_err());
    assert_msg_present_in_logs(res.unwrap_err(), "invalid account data for instruction");
}

#[tokio::test]
#[rstest::rstest]
#[case(spl_token::id())]
#[case(spl_token_2022::id())]
async fn test_add_spl_gas_invalid_config_pda_ata_wrong_owner(#[case] token_program_id: Pubkey) {
    let pt = ProgramTest::default();
    let mut test_fixture = TestFixture::new(pt).await;
    let gas_utils = test_fixture.deploy_gas_service().await;
    test_fixture.init_gas_config(&gas_utils).await.unwrap();

    // Setup mint
    let payer = Keypair::new();
    let wrong_owner = Keypair::new();
    let mint_authority = Keypair::new();
    let decimals = 10;

    let mint = test_fixture
        .init_new_mint(mint_authority.pubkey(), token_program_id, decimals)
        .await;

    // Create payer's ATA
    let payer_ata = test_fixture
        .init_associated_token_account(&mint, &payer.pubkey(), &token_program_id)
        .await;

    let gas_amount = 1_000_000;
    test_fixture
        .mint_tokens_to(
            &mint,
            &payer_ata,
            &mint_authority,
            gas_amount,
            &token_program_id,
        )
        .await;

    // Create ATA for a different owner instead of config_pda
    let wrong_owner_ata = test_fixture
        .init_associated_token_account(&mint, &wrong_owner.pubkey(), &token_program_id)
        .await;

    // Create the instruction with wrong owner's ATA as config_pda_ata
    let refund_address = Pubkey::new_unique();
    let tx_hash = [42; 64];
    let ix_index = 1;
    let event_ix_index = 2;

    let mut ix = axelar_solana_gas_service::instructions::add_spl_gas_instruction(
        &payer.pubkey(),
        &payer_ata,
        &mint,
        &token_program_id,
        &[],
        tx_hash,
        ix_index,
        event_ix_index,
        gas_amount,
        refund_address,
        decimals,
    )
    .unwrap();

    // Replace the config_pda_ata with wrong owner's ATA
    ix.accounts[3].pubkey = wrong_owner_ata;

    // Send transaction (should fail)
    let res = test_fixture
        .send_tx_with_custom_signers(&[ix], &[&test_fixture.payer.insecure_clone(), &payer])
        .await;

    // Assert that the transaction failed with InvalidAccountData error
    assert!(res.is_err());
    assert_msg_present_in_logs(res.unwrap_err(), "invalid account data for instruction");
}
