//! Solidity ABI bindings and compiled bytecode for the test contracts.
//!
//! The sources live in `integration/contracts/` and the compiled artifacts in
//! `integration/contracts/artifacts/`. Both are checked into git; run
//! `integration/contracts/build.sh` after changing a `.sol` file.

use alloy::primitives::{Address, U256};
use alloy::sol;
use alloy::sol_types::SolValue;

sol! {
    #[sol(all_derives)]
    "contracts/ICounter.sol"
}

sol! {
    #[sol(all_derives)]
    "contracts/IERC20.sol"
}

sol! {
    #[sol(all_derives)]
    "contracts/ISimpleSwap.sol"
}

sol! {
    /// The open mint of `TestERC20.sol`, which is not part of [`IERC20`].
    #[sol(all_derives)]
    interface ITestERC20 {
        function mint(address to, uint256 value) external returns (bool);
    }
}

/// Which bytecode flavour to deploy.
///
/// The runtime sets `AllowEVMBytecode = true`, so both are accepted. They take
/// different code paths inside `pallet_revive`, so the tests cover both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeKind {
    /// Plain EVM bytecode produced by `solc`.
    #[default]
    Evm,
    /// PolkaVM blob produced by `resolc`.
    PolkaVM,
}

impl CodeKind {
    /// Both flavours, for tests that want to run the same scenario twice.
    pub const ALL: [CodeKind; 2] = [CodeKind::Evm, CodeKind::PolkaVM];
}

/// A compiled test contract.
#[derive(Clone, Copy, Debug)]
pub struct ContractCode {
    /// Contract name, as used for the artifact file names.
    pub name: &'static str,
    /// Hex-encoded EVM creation bytecode (`solc --bin`).
    evm_hex: &'static str,
    /// PolkaVM blob (`resolc --bin`).
    polkavm: &'static [u8],
}

macro_rules! contract_code {
    ($name:literal) => {
        ContractCode {
            name: $name,
            evm_hex: include_str!(concat!("../contracts/artifacts/", $name, ".bin")),
            polkavm: include_bytes!(concat!("../contracts/artifacts/", $name, ".polkavm")),
        }
    };
}

pub const COUNTER: ContractCode = contract_code!("Counter");
pub const TEST_ERC20: ContractCode = contract_code!("TestERC20");
pub const SIMPLE_SWAP: ContractCode = contract_code!("SimpleSwap");

impl ContractCode {
    /// The EVM creation bytecode.
    pub fn evm(&self) -> Vec<u8> {
        alloy::hex::decode(self.evm_hex.trim())
            .unwrap_or_else(|err| panic!("invalid {} artifact: {err}", self.name))
    }

    /// The PolkaVM blob.
    pub fn polkavm(&self) -> Vec<u8> {
        self.polkavm.to_vec()
    }

    /// Splits a deployment into the `(code, data)` pair expected by
    /// `revive.instantiate_with_code`.
    ///
    /// `pallet_revive` treats the two bytecode flavours differently: PolkaVM
    /// blobs take the constructor arguments in `data`, while EVM creation code
    /// follows the usual Ethereum convention of appending the arguments to the
    /// init code and requires `data` to be empty.
    pub fn deploy_payload(&self, kind: CodeKind, ctor_args: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        match kind {
            CodeKind::Evm => {
                let mut code = self.evm();
                code.extend_from_slice(&ctor_args);
                (code, Vec::new())
            }
            CodeKind::PolkaVM => (self.polkavm(), ctor_args),
        }
    }

    /// The full init code for an Ethereum `eth_sendRawTransaction` deployment.
    ///
    /// Ethereum transactions have a single data field, so PolkaVM blobs get the
    /// constructor arguments appended as well.
    pub fn init_code(&self, kind: CodeKind, ctor_args: Vec<u8>) -> Vec<u8> {
        let (mut code, data) = self.deploy_payload(kind, ctor_args);
        code.extend_from_slice(&data);
        code
    }
}

/// ABI-encoded constructor arguments for the test contracts.
pub mod ctor {
    use super::*;

    /// `Counter(uint256 initialValue)`.
    pub fn counter(initial_value: u64) -> Vec<u8> {
        (U256::from(initial_value),).abi_encode_params()
    }

    /// `TestERC20(string name, string symbol)`.
    pub fn test_erc20(name: &str, symbol: &str) -> Vec<u8> {
        (name.to_string(), symbol.to_string()).abi_encode_params()
    }

    /// `SimpleSwap(address a, address b, uint256 rateNum, uint256 rateDen)`.
    pub fn simple_swap(token_a: Address, token_b: Address, rate_num: u128, rate_den: u128) -> Vec<u8> {
        (token_a, token_b, U256::from(rate_num), U256::from(rate_den)).abi_encode_params()
    }
}
