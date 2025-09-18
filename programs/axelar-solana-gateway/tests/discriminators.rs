mod instruction_discriminators {
    use axelar_solana_gateway::discriminators::{
        APPROVE_MESSAGE, CALL_CONTRACT, CLOSE_MESSAGE_PAYLOAD, COMMIT_MESSAGE_PAYLOAD,
        INITIALIZE_CONFIG, INITIALIZE_MESSAGE_PAYLOAD, INITIALIZE_PAYLOAD_VERIFICATION_SESSION,
        ROTATE_SIGNERS, TRANSFER_OPERATORSHIP, VALIDATE_MESSAGE, VERIFY_SIGNATURE,
        WRITE_MESSAGE_PAYLOAD,
    };
    use discriminator_utils::compute_instruction_discriminator;

    #[test]
    fn test_approve_message_discriminator() {
        let approve_message_discriminator = compute_instruction_discriminator("approve_message");

        assert_eq!(approve_message_discriminator, APPROVE_MESSAGE);
    }

    #[test]
    fn test_rotate_signers_discriminator() {
        let rotate_signers_discriminator = compute_instruction_discriminator("rotate_signers");

        assert_eq!(rotate_signers_discriminator, ROTATE_SIGNERS);
    }

    #[test]
    fn test_call_contract_discriminator() {
        let call_contract_discriminator = compute_instruction_discriminator("call_contract");

        assert_eq!(call_contract_discriminator, CALL_CONTRACT);
    }

    #[test]
    fn test_initialize_config_discriminator() {
        let initialize_config_discriminator =
            compute_instruction_discriminator("initialize_config");

        assert_eq!(initialize_config_discriminator, INITIALIZE_CONFIG);
    }

    #[test]
    fn test_initialize_payload_verification_session_discriminator() {
        let initialize_payload_verification_session_discriminator =
            compute_instruction_discriminator("initialize_payload_verification_session");

        assert_eq!(
            initialize_payload_verification_session_discriminator,
            INITIALIZE_PAYLOAD_VERIFICATION_SESSION
        );
    }

    #[test]
    fn test_verify_signature_discriminator() {
        let verify_signature_discriminator = compute_instruction_discriminator("verify_signature");

        assert_eq!(verify_signature_discriminator, VERIFY_SIGNATURE);
    }

    #[test]
    fn test_validate_message_discriminator() {
        let validate_message_discriminator = compute_instruction_discriminator("validate_message");

        assert_eq!(validate_message_discriminator, VALIDATE_MESSAGE);
    }

    #[test]
    fn test_initialize_message_payload_discriminator() {
        let initialize_message_payload_discriminator =
            compute_instruction_discriminator("initialize_message_payload");

        assert_eq!(
            initialize_message_payload_discriminator,
            INITIALIZE_MESSAGE_PAYLOAD
        );
    }

    #[test]
    fn test_write_message_payload_discriminator() {
        let write_message_payload_discriminator =
            compute_instruction_discriminator("write_message_payload");

        assert_eq!(write_message_payload_discriminator, WRITE_MESSAGE_PAYLOAD);
    }

    #[test]
    fn test_commit_message_payload_discriminator() {
        let commit_message_payload_discriminator =
            compute_instruction_discriminator("commit_message_payload");

        assert_eq!(commit_message_payload_discriminator, COMMIT_MESSAGE_PAYLOAD);
    }

    #[test]
    fn test_close_message_payload_discriminator() {
        let close_message_payload_discriminator =
            compute_instruction_discriminator("close_message_payload");

        assert_eq!(close_message_payload_discriminator, CLOSE_MESSAGE_PAYLOAD);
    }

    #[test]
    fn test_transfer_operatorship_discriminator() {
        let transfer_operatorship_discriminator =
            compute_instruction_discriminator("transfer_operatorship");

        assert_eq!(transfer_operatorship_discriminator, TRANSFER_OPERATORSHIP);
    }
}

mod pda_discriminators {
    use axelar_solana_gateway::discriminators::{
        CONFIG_PDA_DISCRIMINATOR, INCOMING_MESSAGE_PDA_DISCRIMINATOR,
        VERIFICATION_SESSION_ACCOUNT_PDA_DISCRIMINATOR, VERIFIER_SET_TRACKER_PDA_DISCRIMINATOR,
    };
    use discriminator_utils::compute_account_discriminator;

    #[test]
    fn test_config_pda_discriminator() {
        let config_pda_discriminator = compute_account_discriminator("GatewayConfig");

        assert_eq!(config_pda_discriminator, CONFIG_PDA_DISCRIMINATOR);
    }

    #[test]
    fn test_verifier_set_tracker_pda_discriminator() {
        let verifier_set_tracker_pda_discriminator =
            compute_account_discriminator("VerifierSetTracker");

        assert_eq!(
            verifier_set_tracker_pda_discriminator,
            VERIFIER_SET_TRACKER_PDA_DISCRIMINATOR
        );
    }

    #[test]
    fn test_incoming_message_pda_discriminator() {
        let incoming_message_pda_discriminator = compute_account_discriminator("IncomingMessage");

        assert_eq!(
            incoming_message_pda_discriminator,
            INCOMING_MESSAGE_PDA_DISCRIMINATOR
        );
    }

    #[test]
    fn test_verification_session_account_pda_discriminator() {
        let verification_session_account_pda_discriminator =
            compute_account_discriminator("VerificationSessionAccount");

        assert_eq!(
            verification_session_account_pda_discriminator,
            VERIFICATION_SESSION_ACCOUNT_PDA_DISCRIMINATOR
        );
    }
}
