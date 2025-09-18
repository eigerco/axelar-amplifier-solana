mod instruction_discriminators {
    use axelar_solana_its::discriminators::{
        ACCEPT_INTERCHAIN_TOKEN_MINTERSHIP, ACCEPT_OPERATORSHIP, ACCEPT_TOKEN_MANAGER_OPERATORSHIP,
        ADD_TOKEN_MANAGER_FLOW_LIMITER, APPROVE_DEPLOY_REMOTE_INTERCHAIN_TOKEN,
        CALL_CONTRACT_WITH_INTERCHAIN_TOKEN, DEPLOY_INTERCHAIN_TOKEN,
        DEPLOY_REMOTE_CANONICAL_INTERCHAIN_TOKEN, DEPLOY_REMOTE_INTERCHAIN_TOKEN,
        DEPLOY_REMOTE_INTERCHAIN_TOKEN_WITH_MINTER, HANDOVER_MINT_AUTHORITY, INITIALIZE,
        INTERCHAIN_TRANSFER, ITS_GMP_PAYLOAD, LINK_TOKEN, MINT_INTERCHAIN_TOKEN,
        PROPOSE_INTERCHAIN_TOKEN_MINTERSHIP, PROPOSE_OPERATORSHIP,
        PROPOSE_TOKEN_MANAGER_OPERATORSHIP, REGISTER_CANONICAL_INTERCHAIN_TOKEN,
        REGISTER_CUSTOM_TOKEN, REGISTER_TOKEN_METADATA, REMOVE_TOKEN_MANAGER_FLOW_LIMITER,
        REMOVE_TRUSTED_CHAIN, REVOKE_DEPLOY_REMOTE_INTERCHAIN_TOKEN, SET_FLOW_LIMIT,
        SET_PAUSE_STATUS, SET_TOKEN_MANAGER_FLOW_LIMIT, SET_TRUSTED_CHAIN,
        TRANSFER_INTERCHAIN_TOKEN_MINTERSHIP, TRANSFER_OPERATORSHIP,
        TRANSFER_TOKEN_MANAGER_OPERATORSHIP,
    };
    use discriminator_utils::compute_instruction_discriminator;

    #[test]
    fn test_mint_interchain_token_discriminator() {
        let mint_interchain_token_discriminator =
            compute_instruction_discriminator("mint_interchain_token");

        assert_eq!(mint_interchain_token_discriminator, MINT_INTERCHAIN_TOKEN);
    }

    #[test]
    fn test_transfer_interchain_token_mintership_discriminator() {
        let transfer_interchain_token_mintership_discriminator =
            compute_instruction_discriminator("transfer_interchain_token_mintership");

        assert_eq!(
            transfer_interchain_token_mintership_discriminator,
            TRANSFER_INTERCHAIN_TOKEN_MINTERSHIP
        );
    }

    #[test]
    fn test_propose_interchain_token_mintership_discriminator() {
        let propose_interchain_token_mintership_discriminator =
            compute_instruction_discriminator("propose_interchain_token_mintership");

        assert_eq!(
            propose_interchain_token_mintership_discriminator,
            PROPOSE_INTERCHAIN_TOKEN_MINTERSHIP
        );
    }

    #[test]
    fn test_accept_interchain_token_mintership_discriminator() {
        let accept_interchain_token_mintership_discriminator =
            compute_instruction_discriminator("accept_interchain_token_mintership");

        assert_eq!(
            accept_interchain_token_mintership_discriminator,
            ACCEPT_INTERCHAIN_TOKEN_MINTERSHIP
        );
    }

    #[test]
    fn test_set_token_manager_flow_limit_discriminator() {
        let set_token_manager_flow_limit_discriminator =
            compute_instruction_discriminator("set_token_manager_flow_limit");

        assert_eq!(
            set_token_manager_flow_limit_discriminator,
            SET_TOKEN_MANAGER_FLOW_LIMIT
        );
    }

    #[test]
    fn test_add_token_manager_flow_limiter_discriminator() {
        let add_token_manager_flow_limiter_discriminator =
            compute_instruction_discriminator("add_token_manager_flow_limiter");

        assert_eq!(
            add_token_manager_flow_limiter_discriminator,
            ADD_TOKEN_MANAGER_FLOW_LIMITER
        );
    }

    #[test]
    fn test_remove_token_manager_flow_limiter_discriminator() {
        let remove_token_manager_flow_limiter_discriminator =
            compute_instruction_discriminator("remove_token_manager_flow_limiter");

        assert_eq!(
            remove_token_manager_flow_limiter_discriminator,
            REMOVE_TOKEN_MANAGER_FLOW_LIMITER
        );
    }

    #[test]
    fn test_transfer_token_manager_operatorship_discriminator() {
        let transfer_token_manager_operatorship_discriminator =
            compute_instruction_discriminator("transfer_token_manager_operatorship");

        assert_eq!(
            transfer_token_manager_operatorship_discriminator,
            TRANSFER_TOKEN_MANAGER_OPERATORSHIP
        );
    }

    #[test]
    fn test_propose_token_manager_operatorship_discriminator() {
        let propose_token_manager_operatorship_discriminator =
            compute_instruction_discriminator("propose_token_manager_operatorship");

        assert_eq!(
            propose_token_manager_operatorship_discriminator,
            PROPOSE_TOKEN_MANAGER_OPERATORSHIP
        );
    }

    #[test]
    fn test_accept_token_manager_operatorship_discriminator() {
        let accept_token_manager_operatorship_discriminator =
            compute_instruction_discriminator("accept_token_manager_operatorship");

        assert_eq!(
            accept_token_manager_operatorship_discriminator,
            ACCEPT_TOKEN_MANAGER_OPERATORSHIP
        );
    }

    #[test]
    fn test_handover_mint_authority_discriminator() {
        let handover_mint_authority_discriminator =
            compute_instruction_discriminator("handover_mint_authority");

        assert_eq!(
            handover_mint_authority_discriminator,
            HANDOVER_MINT_AUTHORITY
        );
    }

    #[test]
    fn test_initialize_discriminator() {
        let initialize_discriminator = compute_instruction_discriminator("initialize");

        assert_eq!(initialize_discriminator, INITIALIZE);
    }

    #[test]
    fn test_set_pause_status_discriminator() {
        let set_pause_status_discriminator = compute_instruction_discriminator("set_pause_status");

        assert_eq!(set_pause_status_discriminator, SET_PAUSE_STATUS);
    }

    #[test]
    fn test_set_trusted_chain_discriminator() {
        let set_trusted_chain_discriminator =
            compute_instruction_discriminator("set_trusted_chain");

        assert_eq!(set_trusted_chain_discriminator, SET_TRUSTED_CHAIN);
    }

    #[test]
    fn test_remove_trusted_chain_discriminator() {
        let remove_trusted_chain_discriminator =
            compute_instruction_discriminator("remove_trusted_chain");

        assert_eq!(remove_trusted_chain_discriminator, REMOVE_TRUSTED_CHAIN);
    }

    #[test]
    fn test_approve_deploy_remote_interchain_token_discriminator() {
        let approve_deploy_remote_interchain_token_discriminator =
            compute_instruction_discriminator("approve_deploy_remote_interchain_token");

        assert_eq!(
            approve_deploy_remote_interchain_token_discriminator,
            APPROVE_DEPLOY_REMOTE_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_revoke_deploy_remote_interchain_token_discriminator() {
        let revoke_deploy_remote_interchain_token_discriminator =
            compute_instruction_discriminator("revoke_deploy_remote_interchain_token");

        assert_eq!(
            revoke_deploy_remote_interchain_token_discriminator,
            REVOKE_DEPLOY_REMOTE_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_register_canonical_interchain_token_discriminator() {
        let register_canonical_interchain_token_discriminator =
            compute_instruction_discriminator("register_canonical_interchain_token");

        assert_eq!(
            register_canonical_interchain_token_discriminator,
            REGISTER_CANONICAL_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_deploy_remote_canonical_interchain_token_discriminator() {
        let deploy_remote_canonical_interchain_token_discriminator =
            compute_instruction_discriminator("deploy_remote_canonical_interchain_token");

        assert_eq!(
            deploy_remote_canonical_interchain_token_discriminator,
            DEPLOY_REMOTE_CANONICAL_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_deploy_interchain_token_discriminator() {
        let deploy_interchain_token_discriminator =
            compute_instruction_discriminator("deploy_interchain_token");

        assert_eq!(
            deploy_interchain_token_discriminator,
            DEPLOY_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_deploy_remote_interchain_token_discriminator() {
        let deploy_remote_interchain_token_discriminator =
            compute_instruction_discriminator("deploy_remote_interchain_token");

        assert_eq!(
            deploy_remote_interchain_token_discriminator,
            DEPLOY_REMOTE_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_deploy_remote_interchain_token_with_minter_discriminator() {
        let deploy_remote_interchain_token_with_minter_discriminator =
            compute_instruction_discriminator("deploy_remote_interchain_token_with_minter");

        assert_eq!(
            deploy_remote_interchain_token_with_minter_discriminator,
            DEPLOY_REMOTE_INTERCHAIN_TOKEN_WITH_MINTER
        );
    }

    #[test]
    fn test_register_token_metadata_discriminator() {
        let register_token_metadata_discriminator =
            compute_instruction_discriminator("register_token_metadata");

        assert_eq!(
            register_token_metadata_discriminator,
            REGISTER_TOKEN_METADATA
        );
    }

    #[test]
    fn test_register_custom_token_discriminator() {
        let register_custom_token_discriminator =
            compute_instruction_discriminator("register_custom_token");

        assert_eq!(register_custom_token_discriminator, REGISTER_CUSTOM_TOKEN);
    }

    #[test]
    fn test_link_token_discriminator() {
        let link_token_discriminator = compute_instruction_discriminator("link_token");

        assert_eq!(link_token_discriminator, LINK_TOKEN);
    }

    #[test]
    fn test_interchain_transfer_discriminator() {
        let interchain_transfer_discriminator =
            compute_instruction_discriminator("interchain_transfer");

        assert_eq!(interchain_transfer_discriminator, INTERCHAIN_TRANSFER);
    }

    #[test]
    fn test_call_contract_with_interchain_token_discriminator() {
        let call_contract_with_interchain_token_discriminator =
            compute_instruction_discriminator("call_contract_with_interchain_token");

        assert_eq!(
            call_contract_with_interchain_token_discriminator,
            CALL_CONTRACT_WITH_INTERCHAIN_TOKEN
        );
    }

    #[test]
    fn test_set_flow_limit_discriminator() {
        let set_flow_limit_discriminator = compute_instruction_discriminator("set_flow_limit");

        assert_eq!(set_flow_limit_discriminator, SET_FLOW_LIMIT);
    }

    #[test]
    fn test_its_gmp_payload_discriminator() {
        let its_gmp_payload_discriminator = compute_instruction_discriminator("its_gmp_payload");

        assert_eq!(its_gmp_payload_discriminator, ITS_GMP_PAYLOAD);
    }

    #[test]
    fn test_transfer_operatorship_discriminator() {
        let transfer_operatorship_discriminator =
            compute_instruction_discriminator("transfer_operatorship");

        assert_eq!(transfer_operatorship_discriminator, TRANSFER_OPERATORSHIP);
    }

    #[test]
    fn test_propose_operatorship_discriminator() {
        let propose_operatorship_discriminator =
            compute_instruction_discriminator("propose_operatorship");

        assert_eq!(propose_operatorship_discriminator, PROPOSE_OPERATORSHIP);
    }

    #[test]
    fn test_accept_operatorship_discriminator() {
        let accept_operatorship_discriminator =
            compute_instruction_discriminator("accept_operatorship");

        assert_eq!(accept_operatorship_discriminator, ACCEPT_OPERATORSHIP);
    }
}

mod pda_discriminators {
    use axelar_solana_its::discriminators::{
        DEPLOY_APPROVAL_PDA_DISCRIMINATOR, FLOW_STATE_PDA_DISCRIMINATOR,
        INTERCHAIN_TOKEN_SERVICE_PDA_DISCRIMINATOR, TOKEN_MANAGER_PDA_DISCRIMINATOR,
    };
    use discriminator_utils::compute_account_discriminator;

    #[test]
    fn test_interchain_token_service_pda_discriminator() {
        let interchain_token_service_pda_discriminator =
            compute_account_discriminator("InterchainTokenService");

        assert_eq!(
            interchain_token_service_pda_discriminator,
            INTERCHAIN_TOKEN_SERVICE_PDA_DISCRIMINATOR
        );
    }

    #[test]
    fn test_token_manager_pda_discriminator() {
        let token_manager_pda_discriminator = compute_account_discriminator("TokenManager");

        assert_eq!(
            token_manager_pda_discriminator,
            TOKEN_MANAGER_PDA_DISCRIMINATOR,
        );
    }

    #[test]
    fn test_deploy_approval_pda_discriminator() {
        let deploy_approval_pda_discriminator = compute_account_discriminator("DeployApproval");

        assert_eq!(
            deploy_approval_pda_discriminator,
            DEPLOY_APPROVAL_PDA_DISCRIMINATOR
        );
    }

    #[test]
    fn test_flow_state_pda_discriminator() {
        let flow_state_pda_discriminator = compute_account_discriminator("FlowState");

        assert_eq!(flow_state_pda_discriminator, FLOW_STATE_PDA_DISCRIMINATOR);
    }
}
