mod instruction_discriminators {
    use axelar_solana_gas_service::discriminators::{
        ADD_NATIVE_GAS, ADD_SPL_GAS, COLLECT_NATIVE_FEES, COLLECT_SPL_FEES, INIT_CONFIG,
        PAY_NATIVE_FOR_CONTRACT_CALL, PAY_SPL_FOR_CONTRACT_CALL, REFUND_NATIVE_FEES,
        REFUND_SPL_FEES, TRANSFER_OPERATORSHIP,
    };
    use discriminator_utils::compute_instruction_discriminator;

    #[test]
    fn test_init_config_discriminator() {
        let init_config_discriminator = compute_instruction_discriminator("init_config");

        assert_eq!(init_config_discriminator, INIT_CONFIG);
    }

    #[test]
    fn test_transfer_operatorship_discriminator() {
        let transfer_operatorship_discriminator =
            compute_instruction_discriminator("transfer_operatorship");

        assert_eq!(transfer_operatorship_discriminator, TRANSFER_OPERATORSHIP);
    }

    #[test]
    fn test_pay_native_for_contract_call_discriminator() {
        let pay_native_for_contract_call_discriminator =
            compute_instruction_discriminator("pay_native_for_contract_call");

        assert_eq!(
            pay_native_for_contract_call_discriminator,
            PAY_NATIVE_FOR_CONTRACT_CALL
        );
    }

    #[test]
    fn test_add_native_gas_discriminator() {
        let add_native_gas_discriminator = compute_instruction_discriminator("add_native_gas");

        assert_eq!(add_native_gas_discriminator, ADD_NATIVE_GAS);
    }

    #[test]
    fn test_collect_native_fees_discriminator() {
        let collect_native_fees_discriminator =
            compute_instruction_discriminator("collect_native_fees");

        assert_eq!(collect_native_fees_discriminator, COLLECT_NATIVE_FEES);
    }

    #[test]
    fn test_refund_native_fees_discriminator() {
        let refund_native_fees_discriminator =
            compute_instruction_discriminator("refund_native_fees");

        assert_eq!(refund_native_fees_discriminator, REFUND_NATIVE_FEES);
    }

    #[test]
    fn test_pay_spl_for_contract_call_discriminator() {
        let pay_spl_for_contract_call_discriminator =
            compute_instruction_discriminator("pay_spl_for_contract_call");

        assert_eq!(
            pay_spl_for_contract_call_discriminator,
            PAY_SPL_FOR_CONTRACT_CALL
        );
    }

    #[test]
    fn test_add_spl_gas_discriminator() {
        let add_spl_gas_discriminator = compute_instruction_discriminator("add_spl_gas");

        assert_eq!(add_spl_gas_discriminator, ADD_SPL_GAS);
    }

    #[test]
    fn test_collect_spl_fees_discriminator() {
        let collect_spl_fees_discriminator = compute_instruction_discriminator("collect_spl_fees");

        assert_eq!(collect_spl_fees_discriminator, COLLECT_SPL_FEES);
    }

    #[test]
    fn test_refund_spl_fees_discriminator() {
        let refund_spl_fees_discriminator = compute_instruction_discriminator("refund_spl_fees");

        assert_eq!(refund_spl_fees_discriminator, REFUND_SPL_FEES);
    }
}

mod pda_dicriminators {
    use axelar_solana_gas_service::discriminators::CONFIG_PDA_DISCRIMINATOR;
    use discriminator_utils::compute_account_discriminator;

    #[test]
    fn test_config_discriminator() {
        let config_pda_discriminator = compute_account_discriminator("Config");

        assert_eq!(config_pda_discriminator, CONFIG_PDA_DISCRIMINATOR);
    }
}
