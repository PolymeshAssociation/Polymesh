//! Autogenerated weights for `polymesh_contracts`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-25, STEPS: `200`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./target/release/polymesh
// benchmark
// -p=polymesh_contracts
// -e=*
// -s=200
// -r
// 10
// --execution
// Wasm
// --wasm-execution
// Compiled
// --output
// polymesh_contracts.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

/// Weight functions for `polymesh_contracts`.
pub struct WeightInfo;
impl polymesh_contracts::WeightInfo for WeightInfo {
	// Storage: Identity KeyToIdentityIds (r:2 w:0)
	// Storage: Identity DidRecords (r:1 w:0)
	// Storage: BaseContracts ContractInfoOf (r:1 w:1)
	// Storage: BaseContracts CodeStorage (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Identity CurrentDid (r:1 w:1)
	// Storage: Permissions CurrentPalletName (r:1 w:1)
	// Storage: Permissions CurrentDispatchableName (r:1 w:1)
	// Storage: Identity IsDidFrozen (r:1 w:0)
	// Storage: Asset CustomTypesInverse (r:1 w:1)
	// Storage: Asset CustomTypeIdSequence (r:1 w:1)
	// Storage: Asset CustomTypes (r:0 w:1)
	fn chain_extension_full(n: u32, ) -> Weight {
		(420_029_000 as Weight)
			// Standard Error: 0
			.saturating_add((8_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(DbWeight::get().reads(12 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	// Storage: Identity KeyToIdentityIds (r:1 w:0)
	// Storage: Identity DidRecords (r:1 w:0)
	// Storage: BaseContracts ContractInfoOf (r:1 w:1)
	// Storage: BaseContracts CodeStorage (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	fn chain_extension_early_exit() -> Weight {
		(167_816_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Identity KeyToIdentityIds (r:1 w:0)
	// Storage: Identity DidRecords (r:1 w:0)
	// Storage: Asset CustomTypesInverse (r:1 w:1)
	// Storage: Asset CustomTypeIdSequence (r:1 w:1)
	// Storage: Asset CustomTypes (r:0 w:1)
	fn basic_runtime_call(n: u32, ) -> Weight {
		(38_199_000 as Weight)
			// Standard Error: 0
			.saturating_add((6_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}

    fn prepare_instantiate_full(_in_len: u32) -> Weight {
		0
	}

	// Storage: Identity KeyToIdentityIds (r:2 w:0)
	// Storage: Identity DidRecords (r:1 w:0)
	// Storage: BaseContracts ContractInfoOf (r:1 w:1)
	// Storage: BaseContracts CodeStorage (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Identity IsDidFrozen (r:1 w:0)
	// Storage: Instance2Group ActiveMembers (r:1 w:0)
	// Storage: Instance2Group InactiveMembers (r:1 w:0)
	// Storage: Identity Claims (r:2 w:0)
	// Storage: System Account (r:2 w:2)
	fn call() -> Weight {
		(212_650_000 as Weight)
			.saturating_add(DbWeight::get().reads(13 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	// Storage: Identity KeyToIdentityIds (r:2 w:1)
	// Storage: Identity DidRecords (r:1 w:1)
	// Storage: MultiSig KeyToMultiSig (r:1 w:0)
	// Storage: BaseContracts CodeStorage (r:1 w:1)
	// Storage: BaseContracts AccountCounter (r:1 w:1)
	// Storage: BaseContracts ContractInfoOf (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Identity IsDidFrozen (r:1 w:0)
	// Storage: Instance2Group ActiveMembers (r:1 w:0)
	// Storage: Instance2Group InactiveMembers (r:1 w:0)
	// Storage: Identity Claims (r:2 w:0)
	// Storage: System Account (r:2 w:2)
	fn instantiate_with_hash(s: u32, ) -> Weight {
		(284_887_000 as Weight)
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	// Storage: Identity KeyToIdentityIds (r:2 w:1)
	// Storage: Identity DidRecords (r:1 w:1)
	// Storage: MultiSig KeyToMultiSig (r:1 w:0)
	// Storage: BaseContracts AccountCounter (r:1 w:1)
	// Storage: BaseContracts ContractInfoOf (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Identity IsDidFrozen (r:1 w:0)
	// Storage: Instance2Group ActiveMembers (r:1 w:0)
	// Storage: Instance2Group InactiveMembers (r:1 w:0)
	// Storage: Identity Claims (r:2 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: BaseContracts CodeStorage (r:1 w:1)
	// Storage: BaseContracts PristineCode (r:0 w:1)
	fn instantiate_with_code(c: u32, s: u32, ) -> Weight {
		(476_550_000 as Weight)
			// Standard Error: 0
			.saturating_add((182_000 as Weight).saturating_mul(c as Weight))
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(DbWeight::get().reads(15 as Weight))
			.saturating_add(DbWeight::get().writes(8 as Weight))
	}
}
