use frame_support::{
    assert_err, assert_ok, dispatch::DispatchResultWithPostInfo, weights::GetDispatchInfo,
    StorageMap,
};
use pallet_contracts::{ContractAddressFor, Gas};
use sp_runtime::{traits::Hash, Perbill};

use crate::{
    ext_builder::MockProtocolBaseFees,
    storage::{account_from, make_account_without_cdd, AccountId, TestStorage},
    ExtBuilder,
};
use codec::Encode;
use hex_literal::hex;
use pallet_balances as balances;
use polymesh_common_utilities::{protocol_fee::ProtocolOp, traits::CddAndFeeDetails};
use polymesh_contracts::{Call as ContractsCall, NonceBasedAddressDeterminer};
use polymesh_primitives::{
    Signatory, SmartExtensionMetadata, SmartExtensionType, TemplateMetadata,
};
use test_client::AccountKeyring;

const GAS_LIMIT: Gas = 10_000_000_000;

type Balances = balances::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type WrapperContracts = polymesh_contracts::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Contracts = pallet_contracts::Module<TestStorage>;
type WrapperContractsError = polymesh_contracts::Error<TestStorage>;

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

fn create_se_template<T>(
    template_creator: AccountId,
    instantiation_fee: u128,
    code_hash: <T::Hashing as Hash>::Output,
    wasm: Vec<u8>,
) where
    T: frame_system::Trait<Hash = sp_core::H256>,
{
    let wasm_length_weight = 3598000000;

    // Set payer in context
    TestStorage::set_payer_context(Some(template_creator));

    // Create smart extension metadata
    let se_meta_data = SmartExtensionMetadata {
        url: None,
        se_type: SmartExtensionType::TransferManager,
        instantiation_fee: instantiation_fee,
        usage_fee: 0,
        description: "This is a transfer manager type contract".into(),
        version: "1.0.0".into(),
    };

    // verify the weight value of the put_code extrinsic.
    let weight_of_extrinsic =
    ContractsCall::<TestStorage>::put_code(se_meta_data.clone(), wasm.clone())
        .get_dispatch_info()
        .weight;
    assert_eq!(wasm_length_weight + 50_000_000, weight_of_extrinsic);

    // Execute `put_code`
    assert_ok!(WrapperContracts::put_code(
        Origin::signed(template_creator),
        se_meta_data.clone(),
        wasm
    ));

    // Expected data provide by the runtime.
    let expected_template_metadata = TemplateMetadata {
        meta_info: se_meta_data,
        owner: template_creator,
        is_freeze: false,
    };

    // Verify the storage
    assert_eq!(
        WrapperContracts::get_template_meta_details(code_hash),
        expected_template_metadata
    );

    // Set payer in context
    TestStorage::set_payer_context(None);
}

fn create_contract_instance<T>(
    instance_creator: AccountId,
    code_hash: <T::Hashing as Hash>::Output,
) -> DispatchResultWithPostInfo
where
    T: frame_system::Trait<Hash = sp_core::H256>,
{
    let input_data = hex!("5EBD88D6");
    // Set payer of the transaction
    TestStorage::set_payer_context(Some(instance_creator));

    // Increment the nonce.
    System::inc_account_nonce(instance_creator);

    // create a instance
    let result = WrapperContracts::instantiate(
        Origin::signed(instance_creator),
        100,
        GAS_LIMIT,
        code_hash,
        input_data.to_vec(),
    );

    // Free up the context
    TestStorage::set_payer_context(None);
    result
}

fn get_wrong_code_hash<T>() -> <T::Hashing as Hash>::Output
where
    T: frame_system::Trait<Hash = sp_core::H256>,
{
    T::Hashing::hash(&b"abc".encode())
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

            // Get the balance of the Alice
            let alice_balance = System::account(alice).data.free;

            create_se_template::<TestStorage>(alice, 0, code_hash, wasm);

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
            assert_eq!(balance_of_gainer, fee_deducted);

            // Free up the context.
            TestStorage::set_payer_context(None);
        });
}

#[test]
fn check_instantiation_functionality() {
    // Build wasm and get code_hash
    let (wasm, code_hash) = compile_module::<TestStorage>("flipper").unwrap();
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(0))
        .set_protocol_base_fees(protocol_fee)
        .build()
        .execute_with(|| {
            let input_data = hex!("5EBD88D6");
            let extrinsic_wrapper_weight = 500_000_000;
            let instantiation_fee = 99999;

            let alice = AccountKeyring::Alice.public();
            // Create Alice account & the identity for her.
            let (alice_signed, _) = make_account_without_cdd(alice).unwrap();

            create_se_template::<TestStorage>(alice, instantiation_fee, code_hash, wasm);

            // Get the balance of the Alice
            let alice_balance = System::account(alice).data.free;

            // Bob will create a instance of it.
            let bob = AccountKeyring::Bob.public();
            // Create Alice account & the identity for her.
            let (bob_signed, _) = make_account_without_cdd(bob).unwrap();

            // Get the balance of the Bob
            let bob_balance = System::account(bob).data.free;

            // create instance of contract
            let result = create_contract_instance::<TestStorage>(bob, code_hash);

            assert_ok!(result);
            // Verify the actual weight of the extrinsic.
            assert!(result.unwrap().actual_weight.unwrap() > extrinsic_wrapper_weight);

            // Verify whether the instantiation fee deducted properly or not.
            // Alice balance should increased by `instantiation_fee` and Bob balance should be decreased by the same amount.
            let new_alice_balance = System::account(alice).data.free;
            let new_bob_balance = System::account(bob).data.free;

            assert_eq!(bob_balance - new_bob_balance, instantiation_fee);
            assert_eq!(alice_balance + instantiation_fee, new_alice_balance);

            // Generate the contract address.
            let flipper_address_1 =
                NonceBasedAddressDeterminer::<TestStorage>::contract_address_for(
                    &code_hash,
                    &input_data.to_vec(),
                    &bob,
                );

            // Check whether the contract creation allowed or not with same constructor data.
            // It should be as contract creation is depend on the nonce of the account.

            let result = create_contract_instance::<TestStorage>(bob, code_hash);
            assert_ok!(result);

            // Generate the contract address.
            let flipper_address_2 =
                NonceBasedAddressDeterminer::<TestStorage>::contract_address_for(
                    &code_hash,
                    &input_data.to_vec(),
                    &bob,
                );

            // verify that contract address is different.
            assert!(flipper_address_1 != flipper_address_2);
        });
}

#[test]
fn allow_network_share_deduction() {
    // Build wasm and get code_hash
    let (wasm, code_hash) = compile_module::<TestStorage>("flipper").unwrap();
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(25))
        .set_protocol_base_fees(protocol_fee)
        .build()
        .execute_with(|| {
            let instantiation_fee = 5000;
            let fee_collector = account_from(5000);
            let alice = AccountKeyring::Alice.public();
            // Create Alice account & the identity for her.
            make_account_without_cdd(alice).unwrap();

            // Bob will create a instance of it.
            let bob = AccountKeyring::Bob.public();
            // Create Alice account & the identity for her.
            make_account_without_cdd(bob).unwrap();

            // Create template of se
            create_se_template::<TestStorage>(alice, instantiation_fee, code_hash, wasm);

            // Get the balance of Alice
            let alice_balance = System::account(alice).data.free;
            // Get Network fee collector balance
            let fee_collector_balance = System::account(fee_collector).data.free;

            // create instance of contract
            assert_ok!(create_contract_instance::<TestStorage>(bob, code_hash));

            // check the fee division
            // 25 % of fee should be consumed by the network and 75% should be transferred to template owner.
            let new_alice_balance = System::account(alice).data.free;
            let new_fee_collector_balance = System::account(fee_collector).data.free;
            // 75% check
            assert_eq!(
                alice_balance.saturating_add(Perbill::from_percent(75) * instantiation_fee),
                new_alice_balance
            );
            // 25% check
            assert_eq!(
                fee_collector_balance.saturating_add(Perbill::from_percent(25) * instantiation_fee),
                new_fee_collector_balance
            );
        });
}

#[test]
fn check_behavior_when_instantiation_fee_changes() {
    // Build wasm and get code_hash
    let (wasm, code_hash) = compile_module::<TestStorage>("flipper").unwrap();

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(30))
        .build()
        .execute_with(|| {
            let instantiation_fee = 5000;
            let fee_collector = account_from(5000);
            let alice = AccountKeyring::Alice.public();
            // Create Alice account & the identity for her.
            make_account_without_cdd(alice).unwrap();

            // Bob will create a instance of it.
            let bob = AccountKeyring::Bob.public();
            // Create Alice account & the identity for her.
            make_account_without_cdd(bob).unwrap();

            // Create template of se
            create_se_template::<TestStorage>(alice, instantiation_fee, code_hash, wasm);

            let new_instantiation_fee = 8000;

            // Change instantiation fee of the template
            // Should fail because provide hash doesn't exists
            assert_err!(
                WrapperContracts::change_instantiation_fee(
                    Origin::signed(alice),
                    get_wrong_code_hash::<TestStorage>(),
                    new_instantiation_fee
                ),
                WrapperContractsError::TemplateNotExists
            );

            // Should fail as sender is not the template owner
            assert_err!(
                WrapperContracts::change_instantiation_fee(
                    Origin::signed(AccountKeyring::Dave.public()),
                    code_hash,
                    new_instantiation_fee
                ),
                WrapperContractsError::UnAuthorizedOrigin
            );

            // Should success fully change the instantiation fee
            assert_ok!(WrapperContracts::change_instantiation_fee(
                Origin::signed(alice),
                code_hash,
                new_instantiation_fee
            ));

            // Verify the storage changes
            assert_eq!(
                WrapperContracts::get_template_meta_details(code_hash)
                    .meta_info
                    .instantiation_fee,
                new_instantiation_fee
            );

            // Get the balance of Alice
            let alice_balance = System::account(alice).data.free;
            // Get Network fee collector balance
            let fee_collector_balance = System::account(fee_collector).data.free;

            // create instance of contract
            assert_ok!(create_contract_instance::<TestStorage>(bob, code_hash));

            // check the fee division
            // 30 % of fee should be consumed by the network and 70% should be transferred to template owner.
            let new_alice_balance = System::account(alice).data.free;
            let new_fee_collector_balance = System::account(fee_collector).data.free;
            // 70% check
            assert_eq!(
                alice_balance.saturating_add(Perbill::from_percent(70) * new_instantiation_fee),
                new_alice_balance
            );
            // 30% check
            assert_eq!(
                fee_collector_balance
                    .saturating_add(Perbill::from_percent(30) * new_instantiation_fee),
                new_fee_collector_balance
            );
        });
}

#[test]
fn check_freeze_unfreeze_functionality() {
    // Build wasm and get code_hash
    let (wasm, code_hash) = compile_module::<TestStorage>("flipper").unwrap();

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(30))
        .build()
        .execute_with(|| {
            let instantiation_fee = 5000;
            let alice = AccountKeyring::Alice.public();
            // Create Alice account & the identity for her.
            let (alice_signed, _) = make_account_without_cdd(alice).unwrap();

            // Bob will create a instance of it.
            let bob = AccountKeyring::Bob.public();
            // Create Alice account & the identity for her.
            make_account_without_cdd(bob).unwrap();

            // Create template of se
            create_se_template::<TestStorage>(alice, instantiation_fee, code_hash, wasm);

            // Check whether freeze functionality is working or not
            // successfully freeze the instantiation of the SE template
            assert_ok!(WrapperContracts::freeze_instantiation(
                alice_signed.clone(),
                code_hash
            ));

            // Verify the storage
            assert!(WrapperContracts::get_template_meta_details(code_hash).is_freeze);

            // Should fail when trying to freeze the template again
            assert_err!(
                WrapperContracts::freeze_instantiation(alice_signed.clone(), code_hash),
                WrapperContractsError::InstantiationAlreadyFreezed
            );

            // Instantiation should fail
            assert_err!(
                create_contract_instance::<TestStorage>(bob, code_hash),
                WrapperContractsError::InstantiationIsNotAllowed
            );

            // check unfreeze functionality

            // successfully unfreeze the instantiation of the SE template
            assert_ok!(WrapperContracts::unfreeze_instantiation(
                alice_signed.clone(),
                code_hash
            ));

            // Verify the storage
            assert!(!WrapperContracts::get_template_meta_details(code_hash).is_freeze);

            // Should fail when trying to unfreeze the template again
            assert_err!(
                WrapperContracts::unfreeze_instantiation(alice_signed, code_hash),
                WrapperContractsError::InstantiationAlreadyUnFreezed
            );

            // Instantiation should passed
            assert_ok!(create_contract_instance::<TestStorage>(bob, code_hash));
        });
}
