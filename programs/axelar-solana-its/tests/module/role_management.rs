use borsh::BorshDeserialize;
use solana_program::instruction::AccountMeta;
use solana_program_test::tokio;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use test_context::test_context;

use axelar_solana_gateway_test_fixtures::base::FindLog;
use axelar_solana_its::{
    instruction::InterchainTokenServiceInstruction, state::token_manager::TokenManager, Roles,
};
use role_management::state::UserRoles;

use crate::ItsTestContext;

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_operator_transfer(ctx: &mut ItsTestContext) {
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let bob = Keypair::new();

    let transfer_role_ix =
        axelar_solana_its::instruction::transfer_operatorship(ctx.solana_wallet, bob.pubkey())
            .unwrap();

    ctx.send_solana_tx(&[transfer_role_ix]).await.unwrap();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &its_root_pda,
        &bob.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;

    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(bob_roles.contains(Roles::OPERATOR));

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &its_root_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(!payer_roles.contains(Roles::OPERATOR));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_fail_transfer_when_not_holder(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let alice = Keypair::new();

    // We don't have the role, so this should fail
    let transfer_role_ix =
        axelar_solana_its::instruction::transfer_operatorship(bob.pubkey(), alice.pubkey())
            .unwrap();

    let payer = ctx.solana_chain.fixture.payer.insecure_clone();
    let tx_metadata = ctx
        .solana_chain
        .send_tx_with_custom_signers(
            &[transfer_role_ix],
            &[
                bob.insecure_clone(),
                payer, // The test fixture always uses this as the tx payer, so we need to sign
                       // with this.
            ],
        )
        .await
        .unwrap_err();

    assert!(tx_metadata
        .find_log("User roles account not found")
        .is_some());
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_proposal_acceptance(ctx: &mut ItsTestContext) {
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let bob = Keypair::new();

    let roles_to_transfer = Roles::OPERATOR;

    let proposal_ix =
        axelar_solana_its::instruction::propose_operatorship(ctx.solana_wallet, bob.pubkey())
            .unwrap();

    ctx.send_solana_tx(&[proposal_ix]).await.unwrap();

    let (alice_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &its_root_pda,
        &ctx.solana_wallet,
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&alice_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let alice_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    // Alice should still have the roles
    assert!(alice_roles.contains(roles_to_transfer));

    let accept_ix =
        axelar_solana_its::instruction::accept_operatorship(bob.pubkey(), ctx.solana_wallet)
            .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[
                // First transfer funds to bob so he can pay for the user role account
                system_instruction::transfer(
                    &ctx.solana_chain.fixture.payer.pubkey(),
                    &bob.pubkey(),
                    u32::MAX.into(),
                ),
                accept_ix,
            ],
            &[
                &bob.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await;

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&alice_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let new_alice_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    // Alice should not have the roles anymore
    assert!(!new_alice_roles.contains(roles_to_transfer));

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &its_root_pda,
        &bob.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    // Bob should have the roles now
    assert!(bob_roles.contains(roles_to_transfer));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_add_and_remove_flow_limiter(ctx: &mut ItsTestContext) {
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);
    let bob = Keypair::new();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &bob.pubkey(),
    );

    let add_flow_limiter_ix = axelar_solana_its::instruction::token_manager::add_flow_limiter(
        ctx.solana_chain.fixture.payer.pubkey(),
        token_id,
        bob.pubkey(),
    )
    .unwrap();

    ctx.send_solana_tx(&[add_flow_limiter_ix]).await.unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;

    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    // Bob should have the role now
    assert!(bob_roles.contains(Roles::FLOW_LIMITER));

    let remove_flow_limiter_ix =
        axelar_solana_its::instruction::token_manager::remove_flow_limiter(
            ctx.solana_chain.fixture.payer.pubkey(),
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[remove_flow_limiter_ix]).await.unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;

    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    // Bob should not have the role again
    assert!(!bob_roles.contains(Roles::FLOW_LIMITER));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_token_manager_operator_transfer(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);

    ctx.send_solana_tx(&[
        // First transfer funds to bob so he can pay for the user role account
        system_instruction::transfer(
            &ctx.solana_chain.fixture.payer.pubkey(),
            &bob.pubkey(),
            u16::MAX.into(),
        ),
    ])
    .await;

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::OPERATOR));

    let transfer_operatorship_ix =
        axelar_solana_its::instruction::token_manager::transfer_operatorship(
            ctx.solana_chain.fixture.payer.pubkey(),
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[transfer_operatorship_ix])
        .await
        .unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &bob.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(bob_roles.contains(Roles::OPERATOR));
    assert!(!payer_roles.contains(Roles::OPERATOR));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_token_manager_operator_proposal_acceptance(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);

    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &bob.pubkey(),
        u32::MAX.into(),
    )])
    .await;

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::OPERATOR));

    let propose_operatorship_ix =
        axelar_solana_its::instruction::token_manager::propose_operatorship(
            ctx.solana_chain.fixture.payer.pubkey(),
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[propose_operatorship_ix])
        .await
        .unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::OPERATOR));

    let accept_operatorship_ix =
        axelar_solana_its::instruction::token_manager::accept_operatorship(
            bob.pubkey(),
            token_id,
            ctx.solana_chain.fixture.payer.pubkey(),
        )
        .unwrap();

    let payer_keys = ctx.solana_chain.fixture.payer.insecure_clone();
    ctx.solana_chain
        .send_tx_with_custom_signers(
            &[accept_operatorship_ix],
            &[bob.insecure_clone(), payer_keys],
        )
        .await
        .unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &bob.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(!payer_roles.contains(Roles::OPERATOR));
    assert!(bob_roles.contains(Roles::OPERATOR));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_token_manager_minter_transfer(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::MINTER));

    let transfer_mintership_ix =
        axelar_solana_its::instruction::interchain_token::transfer_mintership(
            ctx.solana_chain.fixture.payer.pubkey(),
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[transfer_mintership_ix]).await.unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &bob.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(!payer_roles.contains(Roles::MINTER));
    assert!(bob_roles.contains(Roles::MINTER));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_successful_token_manager_minter_proposal_acceptance(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);

    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &bob.pubkey(),
        u32::MAX.into(),
    )])
    .await;

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::MINTER));

    let propose_mintership_ix =
        axelar_solana_its::instruction::interchain_token::propose_mintership(
            ctx.solana_chain.fixture.payer.pubkey(),
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[propose_mintership_ix]).await.unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::MINTER));

    let accept_mintership_ix = axelar_solana_its::instruction::interchain_token::accept_mintership(
        bob.pubkey(),
        token_id,
        ctx.solana_chain.fixture.payer.pubkey(),
    )
    .unwrap();

    let payer_keys = ctx.solana_chain.fixture.payer.insecure_clone();
    ctx.solana_chain
        .send_tx_with_custom_signers(&[accept_mintership_ix], &[bob.insecure_clone(), payer_keys])
        .await
        .unwrap();

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    let (bob_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &bob.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(!payer_roles.contains(Roles::MINTER));
    assert!(bob_roles.contains(Roles::MINTER));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_fail_token_manager_minter_proposal_acceptance(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_id);

    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &bob.pubkey(),
        u32::MAX.into(),
    )])
    .await;

    let (payer_roles_pda, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_manager_pda,
        &ctx.solana_chain.fixture.payer.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::MINTER));

    // Trying to accept role that wasn't proposed should fail
    let accept_mintership_ix = axelar_solana_its::instruction::interchain_token::accept_mintership(
        bob.pubkey(),
        token_id,
        ctx.solana_chain.fixture.payer.pubkey(),
    )
    .unwrap();

    let payer_keys = ctx.solana_chain.fixture.payer.insecure_clone();
    let tx_metadata = ctx
        .solana_chain
        .send_tx_with_custom_signers(&[accept_mintership_ix], &[bob.insecure_clone(), payer_keys])
        .await
        .unwrap_err();

    assert!(tx_metadata
        .find_log("Warning: failed to deserialize account as role_management::state::RoleProposal<axelar_solana_its::Roles>: Unexpected length of input. The account might not have been initialized.")
        .is_some());

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&payer_roles_pda, &axelar_solana_its::id())
        .await
        .data;
    let payer_roles = UserRoles::<Roles>::try_from_slice(&data).unwrap();

    assert!(payer_roles.contains(Roles::MINTER));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_fail_mint_without_minter_role(ctx: &mut ItsTestContext) {
    let bob = Keypair::new();
    let token_id = ctx.deployed_interchain_token;
    let (its_root_config_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_manager_pda, _) = axelar_solana_its::find_token_manager_pda(
        &its_root_config_pda,
        &ctx.deployed_interchain_token,
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&token_manager_pda, &axelar_solana_its::id())
        .await
        .data;

    let token_manager = TokenManager::try_from_slice(&data).unwrap();
    let token_address = token_manager.token_address;

    let ata = get_associated_token_address_with_program_id(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &token_address,
        &spl_token_2022::id(),
    );

    let create_token_account_ix = create_associated_token_account(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &ctx.solana_chain.fixture.payer.pubkey(),
        &token_address,
        &spl_token_2022::id(),
    );

    // Transfer minter role to bob so we don't have it anymore
    let transfer_mintership_ix =
        axelar_solana_its::instruction::interchain_token::transfer_mintership(
            ctx.solana_wallet,
            token_id,
            bob.pubkey(),
        )
        .unwrap();

    ctx.send_solana_tx(&[transfer_mintership_ix, create_token_account_ix])
        .await
        .unwrap();

    let mint_ix = axelar_solana_its::instruction::interchain_token::mint(
        token_id,
        token_address,
        ata,
        ctx.solana_chain.fixture.payer.pubkey(),
        spl_token_2022::id(),
        8000_u64,
    )
    .unwrap();

    let tx_metadata = ctx.send_solana_tx(&[mint_ix]).await.unwrap_err();

    assert!(tx_metadata
        .find_log("User doesn't have the required roles")
        .is_some());
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_set_trusted_chain_with_upgrade_authority(ctx: &mut ItsTestContext) {
    let chain_name = "new-chain".to_string();

    // Transfer funds to upgrade authority so they can pay for transactions
    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &ctx.solana_chain.upgrade_authority.pubkey(),
        u32::MAX.into(),
    )])
    .await
    .unwrap();

    let set_trusted_chain_ix = axelar_solana_its::instruction::set_trusted_chain(
        ctx.solana_chain.upgrade_authority.pubkey(),
        chain_name.clone(),
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &ctx.solana_chain.upgrade_authority.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Verify the chain was added as trusted
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_set_trusted_chain_with_operator_role(ctx: &mut ItsTestContext) {
    let chain_name = "operator-chain".to_string();
    let bob = Keypair::new();

    // Transfer funds to bob so he can pay for transactions
    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &bob.pubkey(),
        u32::MAX.into(),
    )])
    .await
    .unwrap();

    // Give bob operator role
    let transfer_operatorship_ix =
        axelar_solana_its::instruction::transfer_operatorship(ctx.solana_wallet, bob.pubkey())
            .unwrap();

    ctx.send_solana_tx(&[transfer_operatorship_ix])
        .await
        .unwrap();

    // Bob sets trusted chain using operator role
    let set_trusted_chain_ix =
        axelar_solana_its::instruction::set_trusted_chain(bob.pubkey(), chain_name.clone())
            .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &bob.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Verify the chain was added as trusted
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_set_trusted_chain_failure_without_authority(ctx: &mut ItsTestContext) {
    let chain_name = "unauthorized-chain".to_string();
    let charlie = Keypair::new();

    // Transfer funds to charlie so he can pay for transactions
    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &charlie.pubkey(),
        u32::MAX.into(),
    )])
    .await
    .unwrap();

    // Charlie has neither upgrade authority nor operator role
    let set_trusted_chain_ix =
        axelar_solana_its::instruction::set_trusted_chain(charlie.pubkey(), chain_name.clone())
            .unwrap();

    let tx_metadata = ctx
        .solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &charlie.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap_err();

    // Verify the transaction failed with proper error
    assert!(tx_metadata
        .find_log("Payer is neither upgrade authority nor operator")
        .is_some());

    // Verify the chain was NOT added as trusted
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(!its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_remove_trusted_chain_with_upgrade_authority(ctx: &mut ItsTestContext) {
    let chain_name = "removable-chain".to_string();

    // Transfer funds to upgrade authority
    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &ctx.solana_chain.upgrade_authority.pubkey(),
        u32::MAX.into(),
    )])
    .await
    .unwrap();

    // First add the chain as trusted
    let set_trusted_chain_ix = axelar_solana_its::instruction::set_trusted_chain(
        ctx.solana_chain.upgrade_authority.pubkey(),
        chain_name.clone(),
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &ctx.solana_chain.upgrade_authority.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Verify the chain was added
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();
    assert!(its_root.trusted_chains.contains(&chain_name));

    // Now remove the chain using upgrade authority
    let remove_trusted_chain_ix = axelar_solana_its::instruction::remove_trusted_chain(
        ctx.solana_chain.upgrade_authority.pubkey(),
        chain_name.clone(),
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[remove_trusted_chain_ix],
            &[
                &ctx.solana_chain.upgrade_authority.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Verify the chain was removed
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(!its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_remove_trusted_chain_with_operator_role(ctx: &mut ItsTestContext) {
    let chain_name = "operator-removable-chain".to_string();
    let bob = Keypair::new();

    // Transfer funds to both upgrade authority and bob
    ctx.send_solana_tx(&[
        system_instruction::transfer(
            &ctx.solana_chain.fixture.payer.pubkey(),
            &ctx.solana_chain.upgrade_authority.pubkey(),
            u32::MAX.into(),
        ),
        system_instruction::transfer(
            &ctx.solana_chain.fixture.payer.pubkey(),
            &bob.pubkey(),
            u32::MAX.into(),
        ),
    ])
    .await
    .unwrap();

    // First add the chain as trusted using upgrade authority
    let set_trusted_chain_ix = axelar_solana_its::instruction::set_trusted_chain(
        ctx.solana_chain.upgrade_authority.pubkey(),
        chain_name.clone(),
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &ctx.solana_chain.upgrade_authority.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Give bob operator role
    let transfer_operatorship_ix =
        axelar_solana_its::instruction::transfer_operatorship(ctx.solana_wallet, bob.pubkey())
            .unwrap();

    ctx.send_solana_tx(&[transfer_operatorship_ix])
        .await
        .unwrap();

    // Bob removes the chain using operator role
    let remove_trusted_chain_ix =
        axelar_solana_its::instruction::remove_trusted_chain(bob.pubkey(), chain_name.clone())
            .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[remove_trusted_chain_ix],
            &[
                &bob.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Verify the chain was removed
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(!its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_remove_trusted_chain_failure_without_authority(ctx: &mut ItsTestContext) {
    let chain_name = "protected-chain".to_string();
    let charlie = Keypair::new();

    // Transfer funds to upgrade authority and charlie
    ctx.send_solana_tx(&[
        system_instruction::transfer(
            &ctx.solana_chain.fixture.payer.pubkey(),
            &ctx.solana_chain.upgrade_authority.pubkey(),
            u32::MAX.into(),
        ),
        system_instruction::transfer(
            &ctx.solana_chain.fixture.payer.pubkey(),
            &charlie.pubkey(),
            u32::MAX.into(),
        ),
    ])
    .await
    .unwrap();

    // First add the chain as trusted using upgrade authority
    let set_trusted_chain_ix = axelar_solana_its::instruction::set_trusted_chain(
        ctx.solana_chain.upgrade_authority.pubkey(),
        chain_name.clone(),
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[set_trusted_chain_ix],
            &[
                &ctx.solana_chain.upgrade_authority.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    // Charlie has neither upgrade authority nor operator role
    let remove_trusted_chain_ix =
        axelar_solana_its::instruction::remove_trusted_chain(charlie.pubkey(), chain_name.clone())
            .unwrap();

    let tx_metadata = ctx
        .solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[remove_trusted_chain_ix],
            &[
                &charlie.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap_err();

    // Verify the transaction failed with proper error
    assert!(tx_metadata
        .find_log("Payer is neither upgrade authority nor operator")
        .is_some());

    // Verify the chain was NOT removed
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&its_root_pda, &axelar_solana_its::id())
        .await
        .data;

    let its_root = axelar_solana_its::state::InterchainTokenService::try_from_slice(&data).unwrap();

    assert!(its_root.trusted_chains.contains(&chain_name));
}

#[test_context(ItsTestContext)]
#[tokio::test]
async fn test_prevent_privilege_escalation_through_different_token(ctx: &mut ItsTestContext) {
    // Alice is our ctx.solana_chain.fixture.payer
    // Create Bob who will be the Flow Limiter
    let bob = Keypair::new();
    let token_a_id = ctx.deployed_interchain_token;
    let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();
    let (token_a_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_a_id);

    // Fund Bob's account so he can pay for transactions
    ctx.send_solana_tx(&[system_instruction::transfer(
        &ctx.solana_chain.fixture.payer.pubkey(),
        &bob.pubkey(),
        u32::MAX.into(),
    )])
    .await
    .unwrap();

    // Alice gives Bob Flow Limiter role on TokenA
    let add_flow_limiter_ix = axelar_solana_its::instruction::token_manager::add_flow_limiter(
        ctx.solana_chain.fixture.payer.pubkey(),
        token_a_id,
        bob.pubkey(),
    )
    .unwrap();

    ctx.send_solana_tx(&[add_flow_limiter_ix]).await.unwrap();

    // Assert that Bob has Flow Limiter role on TokenA
    let (bob_roles_pda_token_a, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_a_manager_pda,
        &bob.pubkey(),
    );
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda_token_a, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles_token_a = UserRoles::<Roles>::try_from_slice(&data).unwrap();
    assert!(bob_roles_token_a.contains(Roles::FLOW_LIMITER));

    // Verify Bob does NOT have Minter role on TokenA yet
    assert!(!bob_roles_token_a.contains(Roles::MINTER));

    // Bob deploys TokenB to become its operator
    let token_b_salt = solana_sdk::keccak::hashv(&[b"salt"]).0;
    let token_b_id = axelar_solana_its::interchain_token_id(&bob.pubkey(), &token_b_salt);
    // Bob attempts to deploy a new token as himself
    let deploy_token_ix = axelar_solana_its::instruction::deploy_interchain_token(
        bob.pubkey(),
        token_b_salt,
        "Token B".to_string(),
        "TOKB".to_string(),
        8,
        0,
        Some(bob.pubkey()), // Bob is the initial minter
    )
    .unwrap();

    ctx.solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[deploy_token_ix],
            &[
                &bob.insecure_clone(),
                &ctx.solana_chain.payer.insecure_clone(),
            ],
        )
        .await
        .unwrap();

    let (token_b_manager_pda, _) =
        axelar_solana_its::find_token_manager_pda(&its_root_pda, &token_b_id);

    // Verify Bob is now an operator on TokenB
    let (bob_roles_pda_token_b, _) = role_management::find_user_roles_pda(
        &axelar_solana_its::id(),
        &token_b_manager_pda,
        &bob.pubkey(),
    );

    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda_token_b, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles_token_b = UserRoles::<Roles>::try_from_slice(&data).unwrap();
    assert!(bob_roles_token_b.contains(Roles::OPERATOR));

    // Bob attempts to exploit the vulnerability to make himself a minter on TokenA
    // The exploit relies on constructing a custom transaction where:
    // - Bob uses his Operator role on TokenB (where he has authority)
    // - But modifies the transfer to target TokenA where he only has Flow Limiter role
    let exploit_ix = {
        let (its_root_pda, _) = axelar_solana_its::find_its_root_pda();

        // Bob's roles on TokenB (where he is Operator)
        let (bob_roles_pda_token_b, _) = role_management::find_user_roles_pda(
            &axelar_solana_its::id(),
            &token_b_manager_pda,
            &bob.pubkey(),
        );

        // Alice's roles on TokenA (where she is Minter)
        let (alice_roles_pda_token_a, _) = role_management::find_user_roles_pda(
            &axelar_solana_its::id(),
            &token_a_manager_pda,
            &ctx.solana_chain.fixture.payer.pubkey(),
        );

        // Bob's roles on TokenA (where he's only Flow Limiter)
        let (bob_roles_pda_token_a, _bob_roles_pda_token_a_bump) =
            role_management::find_user_roles_pda(
                &axelar_solana_its::id(),
                &token_a_manager_pda,
                &bob.pubkey(),
            );

        // Create exploit instruction that uses:
        // - TokenB as the resource/context for authorization (where Bob is Operator)
        // - But transfers Minter role on TokenA from Alice to Bob

        // Create a custom instruction mimicking transfer_mintership instruction
        // with mismatched resource and role accounts
        solana_program::instruction::Instruction {
            program_id: axelar_solana_its::id(),
            accounts: vec![
                AccountMeta::new_readonly(its_root_pda, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new(bob.pubkey(), true),
                // This is where the exploit happens - Bob's roles on TokenB, not TokenA
                AccountMeta::new_readonly(bob_roles_pda_token_b, false),
                // We use resource as TokenB but target roles on TokenA
                AccountMeta::new_readonly(token_b_manager_pda, false),
                AccountMeta::new_readonly(bob.pubkey(), false),
                AccountMeta::new(bob_roles_pda_token_a, false),
                AccountMeta::new_readonly(ctx.solana_chain.fixture.payer.pubkey(), false),
                AccountMeta::new(alice_roles_pda_token_a, false),
            ],
            data: borsh::to_vec(
                &InterchainTokenServiceInstruction::TransferInterchainTokenMintership,
            )
            .unwrap(),
        }
    };

    let tx_metadata = ctx
        .solana_chain
        .fixture
        .send_tx_with_custom_signers(
            &[exploit_ix],
            &[
                &bob.insecure_clone(),
                &ctx.solana_chain.fixture.payer.insecure_clone(), // This is just due to how our
                                                                  // testing fixtures work,
                                                                  // normally Alice wouldn't need
                                                                  // to sign the transaction here.
            ],
        )
        .await
        .unwrap_err();

    // Verify the transaction failed with an error about derived PDA not matching
    // This validates that the fix works and Bob cannot escalate privileges
    assert!(tx_metadata.find_log("Derived PDA").is_some());

    // Ensure that Bob still does not have Minter role on TokenA
    let data = ctx
        .solana_chain
        .fixture
        .get_account(&bob_roles_pda_token_a, &axelar_solana_its::id())
        .await
        .data;
    let bob_roles_token_a = UserRoles::<Roles>::try_from_slice(&data).unwrap();
    assert!(!bob_roles_token_a.contains(Roles::MINTER));
}
