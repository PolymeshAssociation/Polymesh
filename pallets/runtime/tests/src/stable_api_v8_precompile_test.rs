// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_support::{assert_ok, weights::Weight};
use polymesh_runtime_common::Currency;
use sp_core::H160;
use sp_keyring::Sr25519Keyring;

use pallet_revive::precompiles::alloy::{hex, sol_types::SolCall};
use pallet_revive::{precompiles::TransactionLimits, AddressMapper, ExecConfig};

use polymesh_stable_api_precompiles::v8::IPolymeshStableApiV8;

use crate::ext_builder::ExtBuilder;
use crate::storage::{TestStorage, User};

type Revive = pallet_revive::Pallet<TestStorage>;
type Balances = pallet_balances::Pallet<TestStorage>;

/// Precompile address for Stable API v8.
/// Fixed(8) → 0x0000000000000000000000000000000000080000
fn precompile_v8_address() -> H160 {
    H160::from(hex::const_decode_to_array(b"0000000000000000000000000000000000080000").unwrap())
}

/// Helper: call getKeyDid via bare_call and return the decoded bytes32 DID.
fn call_get_key_did(caller: &User, account: H160) -> [u8; 32] {
    let data = IPolymeshStableApiV8::getKeyDidCall {
        account: account.0.into(),
    }
    .abi_encode();

    let data = Revive::bare_call(
        caller.origin(),
        precompile_v8_address(),
        0u32.into(),
        TransactionLimits::WeightAndDeposit {
            weight_limit: Weight::MAX,
            deposit_limit: u128::MAX,
        },
        data,
        ExecConfig::new_substrate_tx(),
    )
    .result
    .unwrap()
    .data;

    IPolymeshStableApiV8::getKeyDidCall::abi_decode_returns(&data)
        .unwrap()
        .0
}

#[test]
fn get_key_did_test() {
    ExtBuilder::default().build().execute_with(|| {
        // Create Alice with a DID and fund + map her account.
        let alice = User::new(Sr25519Keyring::Alice);
        Balances::make_free_balance_be(
            &alice.acc(),
            1_000_000 * polymesh_primitives::constants::currency::POLY,
        );
        assert_ok!(Revive::map_account(alice.origin()));

        // Known address with a DID returns the correct DID.
        let alice_evm =
            <TestStorage as pallet_revive::Config>::AddressMapper::to_address(&alice.acc());
        let ret = call_get_key_did(&alice, alice_evm);
        assert_eq!(
            ret, alice.did.0,
            "alice's EVM address should resolve to her DID"
        );

        // Unknown address returns zero DID.
        let unknown_addr = H160([0xAA; 20]);
        let ret = call_get_key_did(&alice, unknown_addr);
        assert_eq!(ret, [0u8; 32], "unknown address should return zero DID");
    });
}
