#[starknet::contract]
mod get_balance {
    use hdp_cairo::memorizer::account_memorizer::AccountMemorizerTrait;
    use hdp_cairo::{HDP, memorizer::account_memorizer::{AccountKey, AccountMemorizerImpl}};
    use starknet::syscalls::call_contract_syscall;
    use starknet::{ContractAddress, SyscallResult, SyscallResultTrait};

    #[storage]
    struct Storage {}

    #[external(v0)]
    pub fn main(ref self: ContractState, hdp: HDP, block_number: u32, address: felt252) -> u256 {
        hdp
            .account_memorizer
            .get_balance(
                AccountKey { chain_id: 11155111, block_number: block_number.into(), address }
            )
    }
}
