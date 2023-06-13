use crate::{
    ext_builder::ExtBuilder,
    storage::{TestStorage, User},
};
use codec::Encode;
use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop, dispatch::Weight,
};
use polymesh_common_utilities::constants::currency::POLY;
use polymesh_primitives::{AccountId, Gas, Permissions, PortfolioPermissions, Ticker};
use polymesh_runtime_common::Currency;
use sp_runtime::traits::Hash;
use test_client::AccountKeyring;

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

#[test]
fn misc_polymesh_extensions() {
    let eve = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![eve.clone()])
        .adjust(Box::new(move |storage| {
            polymesh_contracts::GenesisConfig {
                call_whitelist: [
                    [0x1A, 0x00],
                    [0x1A, 0x01],
                    [0x1A, 0x02],
                    [0x1A, 0x03],
                    [0x1A, 0x11],
                    [0x2F, 0x01],
                ]
                .into_iter()
                .map(|ext_id: [u8; 2]| ext_id.into())
                .collect(),
            }
            .assimilate_storage(storage)
            .unwrap();
        }))
        .build()
        .execute_with(|| {
            let owner = User::new(AccountKeyring::Alice);
            let user = User::new(AccountKeyring::Bob);
            Balances::make_free_balance_be(&owner.acc(), 1_000_000 * POLY);

            let (code, _) = chain_extension();
            let hash = Hashing::hash(&code);
            let salt = vec![0xFF];

            let perms = Permissions {
                portfolio: PortfolioPermissions::Whole,
                ..Permissions::empty()
            };
            let instantiate = || {
                Contracts::instantiate_with_code_perms(
                    owner.origin(),
                    Balances::minimum_balance(),
                    GAS_LIMIT,
                    None,
                    code.clone(),
                    vec![],
                    salt.clone(),
                    perms.clone(),
                )
            };
            let derive_key = |key, salt| FrameContracts::contract_address(&key, &hash, &[], salt);
            let call = |key: AccountId, value, data| {
                FrameContracts::call(user.origin(), key.into(), value, GAS_LIMIT, None, data)
            };
            let assert_has_secondary_key = |key: AccountId| {
                let data = Identity::get_key_identity_data(key).unwrap();
                assert_eq!(data.identity, owner.did);
                assert_eq!(data.permissions.unwrap(), perms);
            };

            // Instantiate contract.
            // Next time, a secondary key with that key already exists.
            assert_ok!(instantiate());
            assert_noop!(instantiate(), IdentityError::AlreadyLinked);

            // Ensure contract is now a secondary key of Alice.
            let key_first_contract = derive_key(owner.acc(), &salt);
            assert_has_secondary_key(key_first_contract.clone());

            // Ensure a call different non-existent instantiation results in "contract not found".
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(derive_key(owner.acc(), &[0x00]), 0, vec![]),
                BaseContractsError::ContractNotFound,
            ));

            // Execute a chain extension with too long data.
            let call = |value, data| call(key_first_contract.clone(), value, data);
            let mut too_long_data = 0x00_00_00_01.encode();
            too_long_data.extend(vec![b'X'; MaxInLen::get() as usize + 1]);
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(0, too_long_data),
                ContractsError::InLenTooLarge,
            ));

            // Execute a func_id that isn't recognized.
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(0, 0x04_00_00_00.encode()),
                ContractsError::InvalidFuncId,
            ));

            // Input for registering ticker `A` (11 trailing nulls).
            let ticker = Ticker::from_slice_truncated(b"A" as &[u8]);
            let mut register_ticker_data = 0x00_1A_00_00.encode();
            register_ticker_data.extend(ticker.encode());

            // Leave too much data left in the input.
            let mut register_ticker_extra_data = register_ticker_data.clone();
            register_ticker_extra_data.extend(b"X"); // Adding this leaves too much data.
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(0, register_ticker_extra_data),
                ContractsError::DataLeftAfterDecoding,
            ));

            // Execute `register_ticker` but fail due to lacking permissions.
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(0, register_ticker_data.clone()),
                pallet_permissions::Error::<TestStorage>::UnauthorizedCaller,
            ));

            // Grant permissions to `key_first_contract`, and so registration should go through.
            assert_ok!(Identity::set_secondary_key_permissions(
                owner.origin(),
                key_first_contract.clone(),
                Permissions::default(),
            ));

            // The contract doesn't have enough POLYX to cover the protocol fee.
            assert_storage_noop!(assert_err_ignore_postinfo!(
                call(0, register_ticker_data.clone()),
                pallet_protocol_fee::Error::<TestStorage>::InsufficientAccountBalance,
            ));

            // Successfully execute `register_ticker`,
            // ensuring that it was Alice who registered it.
            assert_ok!(call(2500, register_ticker_data));
            assert_ok!(Asset::ensure_owner(&ticker, owner.did));
        })
}
