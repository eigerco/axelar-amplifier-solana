mod instruction_discriminators {
    use axelar_solana_gateway::discriminators::{INITIALIZE_CONFIG, TRANSFER_OPERATORSHIP};
    use axelar_solana_governance::discriminators::{
        EXECUTE_OPERATOR_PROPOSAL, EXECUTE_PROPOSAL, PROCESS_GMP, UPDATE_CONFIG, WITHDRAW_TOKENS,
    };
    use discriminator_utils::compute_instruction_discriminator;

    #[test]
    fn test_transfer_operatorship_discriminator() {
        let transfer_operatorship_discriminator =
            compute_instruction_discriminator("transfer_operatorship");

        assert_eq!(transfer_operatorship_discriminator, TRANSFER_OPERATORSHIP);
    }

    #[test]
    fn test_withdraw_tokens_discriminator() {
        let withdraw_tokens_discriminator = compute_instruction_discriminator("withdraw_tokens");

        assert_eq!(withdraw_tokens_discriminator, WITHDRAW_TOKENS);
    }

    #[test]
    fn test_execute_proposal_discriminator() {
        let execute_proposal_discriminator = compute_instruction_discriminator("execute_proposal");

        assert_eq!(execute_proposal_discriminator, EXECUTE_PROPOSAL);
    }

    #[test]
    fn test_execute_operator_proposal_discriminator() {
        let execute_operator_proposal_discriminator =
            compute_instruction_discriminator("execute_operator_proposal");

        assert_eq!(
            execute_operator_proposal_discriminator,
            EXECUTE_OPERATOR_PROPOSAL
        );
    }

    #[test]
    fn test_initialize_config_discriminator() {
        let initialize_config_discriminator =
            compute_instruction_discriminator("initialize_config");

        assert_eq!(initialize_config_discriminator, INITIALIZE_CONFIG);
    }

    #[test]
    fn test_update_config_discriminator() {
        let update_config_discriminator = compute_instruction_discriminator("update_config");

        assert_eq!(update_config_discriminator, UPDATE_CONFIG);
    }

    #[test]
    fn test_process_gmp_discriminator() {
        let process_gmp_discriminator = compute_instruction_discriminator("process_gmp");

        assert_eq!(process_gmp_discriminator, PROCESS_GMP);
    }
}

mod pda_discriminators {
    use axelar_solana_governance::discriminators::{
        EXECUTABLE_PROPOSAL_PDA_DISCRIMINATOR, GOVERNANCE_CONFIG_PDA_DISCRIMINATOR,
    };
    use discriminator_utils::compute_account_discriminator;

    #[test]
    fn test_governance_config_pda_discriminator() {
        let governance_config_pda_discriminator = compute_account_discriminator("GovernanceConfig");

        assert_eq!(
            governance_config_pda_discriminator,
            GOVERNANCE_CONFIG_PDA_DISCRIMINATOR
        );
    }

    #[test]
    fn test_execute_proposal_pda_discriminator() {
        let execute_proposal_pda_discriminator =
            compute_account_discriminator("ExecutableProposal");

        assert_eq!(
            execute_proposal_pda_discriminator,
            EXECUTABLE_PROPOSAL_PDA_DISCRIMINATOR
        );
    }
}
