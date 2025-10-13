use solana_program::instruction::AccountMeta;

enum RelayerData {
	Bytes(Vec<u8>),
	Message(),
}
enum RelayerAccount {
	Account(AccountMeta),
	IncomingMessage(),
	MessagePayload(),
	Payer(Uint64)
}
struct RelayerInstruction {
	program: Pubkey,
	accounts: Vec<AccountMeta>,
	data: Vec<RelayerData>,
}

struct RelayerTransaction {
	is_final: bool,
	// solana supports a series of instructions in a single transaction,
	// do we want to support that or just use a single instruction?
	instructions: Vec<RelayerInstruction>,
}