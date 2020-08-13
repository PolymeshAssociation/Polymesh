use frame_support::{assert_ok, StorageMap};
use pallet_contracts::{
    BalanceOf, ContractAddressFor, ContractInfo, ContractInfoOf, GenesisConfig, Module,
};
use sp_runtime::{traits::Hash, Perbill};

use crate::{
    ext_builder::MockProtocolBaseFees,
    storage::{account_from, make_account_without_cdd, TestStorage},
    ExtBuilder,
};
use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_common_utilities::{protocol_fee::ProtocolOp, traits::CddAndFeeDetails, Context};
use polymesh_primitives::{
    Signatory, SmartExtensionMetadata, SmartExtensionType, TemplateMetadata,
};

use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type WrapperContracts = polymesh_contracts::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Contracts = pallet_contracts::Module<TestStorage>;

/// Load a given wasm module represented by a .wat file and returns a wasm binary contents along
/// with it's hash.
///
/// The fixture files are located under the `fixtures/` directory.
fn compile_module<T>(
    fixture_name: &str,
) -> Result<(Vec<u8>, <T::Hashing as Hash>::Output), wabt::Error>
where
    T: frame_system::Trait,
{
    use std::fs;

    let fixture_path = ["fixtures/", fixture_name, ".wat"].concat();
    let module_wat_source = fs::read_to_string(&fixture_path)
        .expect(&format!("Unable to find {} fixture", fixture_name));
    let wasm_binary = wabt::wat2wasm(module_wat_source)?;
    let code_hash = T::Hashing::hash(&wasm_binary);
    Ok((wasm_binary, code_hash))
}

#[test]
fn check_put_code_functionality() {
    // Build wasm and get code_hash
    let (wasm, code_hash) = compile_module::<TestStorage>("flipper").unwrap();
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(0))
        .set_protocol_base_fees(protocol_fee)
        .build()
        .execute_with(|| {
            let alice = AccountKeyring::Alice.public();
            // Create Alice account & the identity for her.
            let (alice_signed, _) = make_account_without_cdd(alice).unwrap();

            // Set payer in context
            TestStorage::set_payer_context(Some(Signatory::Account(alice)));

            // Get the balance of the Alice
            let alice_balance = System::account(alice).data.free;

            // Create smart extension metadata
            let se_meta_data = SmartExtensionMetadata {
                url: None,
                se_type: SmartExtensionType::TransferManager,
                instantiation_fee: 0,
                usage_fee: 0,
                description: "This is a transfer manager type contract".into(),
                version: "1.0.0".into(),
            };

            // Execute `put_code`
            assert_ok!(WrapperContracts::put_code(
                alice_signed,
                se_meta_data.clone(),
                wasm
            ));

            // Expected data provide by the runtime.
            let expected_template_metadata = TemplateMetadata {
                meta_info: se_meta_data,
                owner: alice,
                is_freeze: false,
            };

            // Verify the storage
            assert_eq!(
                WrapperContracts::get_template_meta_details(code_hash),
                expected_template_metadata
            );

            // Check the storage of the base pallet
            assert!(<pallet_contracts::PristineCode<TestStorage>>::get(code_hash).is_some());

            // Check for fee
            let fee_deducted = <pallet_protocol_fee::Module<TestStorage>>::compute_fee(
                ProtocolOp::ContractsPutCode,
            );

            // Check for protocol fee deduction
            let current_alice_balance = System::account(alice).data.free;
            assert_eq!(current_alice_balance, alice_balance - fee_deducted);

            // Balance of fee collector
            let balance_of_gainer = System::account(account_from(5000)).data.free;
            assert_eq!(balance_of_gainer, fee_deducted)
        });
}
