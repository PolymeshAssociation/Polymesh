use codec::Encode;
use frame_support::dispatch::{DispatchError, Weight};
use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop, StorageMap,
};
use polymesh_contracts::{
    Api, ApiCodeHash, ApiNextUpgrade, ChainVersion, ExtrinsicId, NextUpgrade,
};
use sp_keyring::AccountKeyring;
use sp_runtime::traits::Hash;

use pallet_identity::ParentDid;
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::{Gas, Permissions, PortfolioPermissions, SubsetRestriction, Ticker};
use polymesh_runtime_common::Currency;

use crate::ext_builder::ExtBuilder;
use crate::storage::{root, TestStorage, User};

// We leave it to tests in the substrate to ensure that `pallet-contracts`
// is functioning correctly, so we do not add such redundant tests
// and instead focus on the particulars of our contracts pallet.
// This includes testing CDD, permissions, and what the chain extension does.

const GAS_LIMIT: Gas = Weight::from_ref_time(100_000_000_000).set_proof_size(3 * 1024 * 1024);

type Asset = pallet_asset::Module<TestStorage>;
type FrameContracts = pallet_contracts::Pallet<TestStorage>;
type BaseContractsError = pallet_contracts::Error<TestStorage>;
type CodeHash = <Hashing as Hash>::Output;
type Hashing = <TestStorage as frame_system::Config>::Hashing;
type Contracts = polymesh_contracts::Pallet<TestStorage>;
type ContractsError = polymesh_contracts::Error<TestStorage>;
type MaxInLen = <TestStorage as polymesh_contracts::Config>::MaxInLen;
type Balances = pallet_balances::Pallet<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type IdentityError = pallet_identity::Error<TestStorage>;
type PolymeshContracts = polymesh_contracts::Pallet<TestStorage>;

/// Load a given wasm module represented by a .wat file
/// and returns a wasm binary contents along with it's hash.
///
/// The fixture files are located under the `fixtures/` directory.
fn compile_module(fixture_name: &str) -> wat::Result<(Vec<u8>, CodeHash)> {
    let fixture_path = ["fixtures/", fixture_name, ".wat"].concat();
    let wasm_binary = wat::parse_file(fixture_path)?;
    let code_hash = Hashing::hash(&wasm_binary);
    Ok((wasm_binary, code_hash))
}

fn chain_extension() -> (Vec<u8>, CodeHash) {
    compile_module("chain_extension").unwrap()
}

fn update_call_runtime_whitelist(extrinsics: Vec<(ExtrinsicId, bool)>) {
    assert_ok!(Contracts::update_call_runtime_whitelist(root(), extrinsics));
}

#[test]
fn deploy_as_secondary_key() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        Balances::make_free_balance_be(&alice.acc(), 1_000_000 * POLY);

        let permissions = Permissions {
            portfolio: PortfolioPermissions::Whole,
            asset: SubsetRestriction::empty(),
            extrinsic: SubsetRestriction::empty(),
        };
        let (code, _) = chain_extension();
        let salt = vec![0xFF];
        // Deploy the contract as a secondary key of Alice
        assert_ok!(Contracts::instantiate_with_code_perms(
            alice.origin(),
            Balances::minimum_balance(),
            GAS_LIMIT,
            None,
            code.clone(),
            vec![],
            salt.clone(),
            permissions.clone(),
        ));
        // Ensures the is a secondary key of alice
        let hash = Hashing::hash(&code);
        let derived_key = FrameContracts::contract_address(&alice.acc(), &hash, &[], &salt);
        let key_data = Identity::get_key_identity_data(derived_key).unwrap();
        assert_eq!(key_data.identity, alice.did);
        assert_eq!(key_data.permissions.unwrap(), permissions);
        // The same contract can't be instantiated twice for the same identity
        assert_noop!(
            Contracts::instantiate_with_code_perms(
                alice.origin(),
                Balances::minimum_balance(),
                GAS_LIMIT,
                None,
                code.clone(),
                vec![],
                salt.clone(),
                permissions.clone(),
            ),
            IdentityError::AlreadyLinked
        );
    })
}

#[test]
fn chain_extension_calls() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"A" as &[u8]);
        Balances::make_free_balance_be(&alice.acc(), 1_000_000 * POLY);

        let permissions = Permissions {
            portfolio: PortfolioPermissions::Whole,
            asset: SubsetRestriction::empty(),
            extrinsic: SubsetRestriction::empty(),
        };
        let (code, _) = chain_extension();
        let salt = vec![0xFF];
        // Deploy the contract as a secondary key of Alice
        assert_ok!(Contracts::instantiate_with_code_perms(
            alice.origin(),
            Balances::minimum_balance(),
            GAS_LIMIT,
            None,
            code.clone(),
            vec![],
            salt.clone(),
            permissions.clone(),
        ));
        // A call to a non-existent instantiation must return an error
        let hash = Hashing::hash(&code);
        let wrong_key = FrameContracts::contract_address(&alice.acc(), &hash, &[], &[0x00]);
        assert_storage_noop!(assert_err_ignore_postinfo!(
            FrameContracts::call(
                bob.origin(),
                wrong_key.into(),
                0,
                GAS_LIMIT,
                None,
                Vec::new()
            ),
            BaseContractsError::ContractNotFound
        ));
        // Calls to functions not recognized must return an error
        let contract_key = FrameContracts::contract_address(&alice.acc(), &hash, &[], &salt);
        assert_storage_noop!(assert_err_ignore_postinfo!(
            FrameContracts::call(
                bob.origin(),
                contract_key.clone().into(),
                0,
                GAS_LIMIT,
                None,
                0x04_00_00_00.encode()
            ),
            ContractsError::InvalidFuncId,
        ));
        // Calls the right chain extension function, but runtime call fails
        let extrinsic_id: ExtrinsicId = [0x1A, 0x00].into();
        update_call_runtime_whitelist(vec![(extrinsic_id.clone(), true)]);
        let register_ticker_input = [
            0x00_00_00_01.encode(),
            extrinsic_id.encode(),
            ticker.encode(),
        ]
        .concat();
        assert_storage_noop!(assert_err_ignore_postinfo!(
            FrameContracts::call(
                bob.origin(),
                contract_key.clone().into(),
                0,
                GAS_LIMIT,
                None,
                register_ticker_input.clone()
            ),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller,
        ));
        // Successfull call
        assert_ok!(Identity::set_secondary_key_permissions(
            alice.origin(),
            contract_key.clone(),
            Permissions::default(),
        ));
        assert_ok!(FrameContracts::call(
            bob.origin(),
            contract_key.into(),
            2_500,
            GAS_LIMIT,
            None,
            register_ticker_input
        ),);
        assert_ok!(Asset::ensure_owner(&ticker, alice.did));
    })
}

#[test]
fn deploy_as_child_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let salt = vec![0xFF];
        let (code, _) = chain_extension();
        let hash = Hashing::hash(&code);
        let alice = User::new(AccountKeyring::Alice);
        Balances::make_free_balance_be(&alice.acc(), 1_000_000 * POLY);

        assert_ok!(Contracts::instantiate_with_code_as_primary_key(
            alice.origin(),
            Balances::minimum_balance(),
            GAS_LIMIT,
            None,
            code.clone(),
            vec![],
            salt.clone(),
        ));

        let contract_account_id = FrameContracts::contract_address(&alice.acc(), &hash, &[], &salt);
        let child_id = Identity::get_identity(&contract_account_id).unwrap();
        assert_eq!(ParentDid::get(child_id), Some(alice.did));
    })
}

#[test]
fn upgrade_api_unauthorized_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let api = Api::new(*b"POLY", 6);
        let chain_version = ChainVersion::new(6, 0);
        let api_code_hash = ApiCodeHash {
            hash: CodeHash::default(),
        };
        let next_upgrade = NextUpgrade::new(chain_version, api_code_hash);

        assert_noop!(
            Contracts::upgrade_api(alice.origin(), api, next_upgrade),
            DispatchError::BadOrigin
        );
    })
}

#[test]
fn upgrade_api() {
    ExtBuilder::default().build().execute_with(|| {
        let api = Api::new(*b"POLY", 6);
        let chain_version = ChainVersion::new(6, 0);
        let api_code_hash = ApiCodeHash {
            hash: CodeHash::default(),
        };
        let next_upgrade = NextUpgrade::new(chain_version, api_code_hash);

        assert_ok!(Contracts::upgrade_api(
            root(),
            api.clone(),
            next_upgrade.clone()
        ));

        assert_eq!(ApiNextUpgrade::get(&api).unwrap(), next_upgrade);
    })
}
