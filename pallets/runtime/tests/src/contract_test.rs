use crate::{
    ext_builder::MockProtocolBaseFees,
    storage::{
        account_from, make_account_with_uid, make_account_without_cdd, AccountId, TestStorage, User,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    weights::GetDispatchInfo,
    StorageMap,
};
use frame_system::RawOrigin;
use hex_literal::hex;
use pallet_balances as balances;
use pallet_contracts::{ContractAddressFor, ContractInfoOf, Gas};
use pallet_permissions as permissions;
use polymesh_common_utilities::{protocol_fee::ProtocolOp, traits::CddAndFeeDetails};
use polymesh_contracts::MetadataOfTemplate;
use polymesh_contracts::{Call as ContractsCall, NonceBasedAddressDeterminer};
use polymesh_primitives::{
    IdentityId, InvestorUid, SmartExtensionType, TemplateDetails, TemplateMetadata,
};
use sp_core::sr25519::Public;
use sp_runtime::{traits::Hash, Perbill};
use test_client::AccountKeyring;

const GAS_LIMIT: Gas = 10_000_000_000;

type Balances = balances::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type WrapperContracts = polymesh_contracts::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Contracts = pallet_contracts::Module<TestStorage>;
type WrapperContractsError = polymesh_contracts::Error<TestStorage>;
type ProtocolFeeError = pallet_protocol_fee::Error<TestStorage>;
type PermissionError = permissions::Error<TestStorage>;
type Hashing = <TestStorage as frame_system::Trait>::Hashing;
type CodeHash = <Hashing as Hash>::Output;

/// Load a given wasm module represented by a .wat file and returns a wasm binary contents along
/// with it's hash.
///
/// The fixture files are located under the `fixtures/` directory.
pub fn compile_module(fixture_name: &str) -> wat::Result<(CodeHash, Vec<u8>)> {
    let wasm_binary = wat::parse_file(["fixtures/", fixture_name, ".wat"].concat())?;
    Ok((Hashing::hash(&wasm_binary), wasm_binary))
}

pub fn flipper() -> (CodeHash, Vec<u8>) {
    compile_module("flipper").unwrap()
}

pub fn create_se_template(
    template_creator: AccountId,
    template_creator_did: IdentityId,
    instantiation_fee: u128,
    code_hash: CodeHash,
    wasm: Vec<u8>,
) {
    let wasm_length_weight = 1426500000;

    // Set payer in context
    TestStorage::set_payer_context(Some(template_creator));

    // Create smart extension metadata
    let se_meta_data = TemplateMetadata {
        url: None,
        se_type: SmartExtensionType::TransferManager,
        usage_fee: 0,
        description: "This is a transfer manager type contract".into(),
        version: 5000,
    };

    // verify the weight value of the put_code extrinsic.
    let weight_of_extrinsic = ContractsCall::<TestStorage>::put_code(
        se_meta_data.clone(),
        instantiation_fee,
        wasm.clone(),
    )
    .get_dispatch_info()
    .weight;
    assert_eq!(wasm_length_weight + 50_000_000, weight_of_extrinsic);

    // Execute `put_code`
    assert_ok!(WrapperContracts::put_code(
        Origin::signed(template_creator),
        se_meta_data.clone(),
        instantiation_fee,
        wasm
    ));

    // Expected data provide by the runtime.
    let expected_template_metadata = TemplateDetails {
        instantiation_fee,
        owner: template_creator_did,
        frozen: false,
    };

    // Verify the storage
    assert_eq!(
        WrapperContracts::get_template_details(code_hash),
        expected_template_metadata
    );

    assert_eq!(WrapperContracts::get_metadata_of(code_hash), se_meta_data);

    // Set payer in context
    TestStorage::set_payer_context(None);
}

pub fn create_contract_instance(
    instance_creator: AccountId,
    code_hash: CodeHash,
    max_fee: u128,
    fail: bool,
) -> DispatchResultWithPostInfo {
    let input_data = hex!("0222FF18");
    // Set payer of the transaction
    TestStorage::set_payer_context(Some(instance_creator));

    // Access the extension nonce.
    let current_extension_nonce = WrapperContracts::extension_nonce();

    // create a instance
    let result = WrapperContracts::instantiate(
        Origin::signed(instance_creator),
        100,
        GAS_LIMIT,
        code_hash,
        input_data.to_vec(),
        max_fee,
    );

    if !fail {
        assert_eq!(
            WrapperContracts::extension_nonce(),
            current_extension_nonce + 1
        );
    }

    // Free up the context
    TestStorage::set_payer_context(None);
    result
}

fn get_wrong_code_hash() -> CodeHash {
    Hashing::hash(&b"abc".encode())
}

/// Executes `f` on a created `TestExternalities` using the given `network_fee_share` and `protocol_base_fees`.
///
/// It also enables the `put_code` extrinsics.
fn execute_externalities_with_wasm(
    network_fee_share: u32,
    protocol_base_fees: MockProtocolBaseFees,
    f: impl FnOnce(Vec<u8>, CodeHash),
) {
    let (code_hash, wasm) = flipper();
    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(network_fee_share))
        .set_protocol_base_fees(protocol_base_fees)
        .set_contracts_put_code(true)
        .build()
        .execute_with(|| f(wasm, code_hash))
}

fn free(acc: AccountId) -> u128 {
    System::account(acc).data.free
}

#[test]
fn check_put_code_functionality() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    execute_externalities_with_wasm(0, protocol_fee, |wasm, code_hash| {
        let alice = AccountKeyring::Alice.public();
        // Create Alice account & the identity for her.
        let (_, alice_did) = make_account_without_cdd(alice).unwrap();

        // Get the balance of the Alice.
        let alice_balance = free(alice);

        create_se_template(alice, alice_did, 0, code_hash, wasm);

        // Check the storage of the base pallet.
        assert!(<pallet_contracts::PristineCode<TestStorage>>::get(code_hash).is_some());

        // Check for fee.
        let fee_deducted = <pallet_protocol_fee::Module<TestStorage>>::compute_fee(&[
            ProtocolOp::ContractsPutCode,
        ]);

        // Check for protocol fee deduction.
        assert_eq!(free(alice), alice_balance - fee_deducted);

        // Balance of fee collector
        assert_eq!(free(account_from(5000)), fee_deducted);

        // Free up the context.
        TestStorage::set_payer_context(None);
    });
}

#[test]
fn check_instantiation_functionality() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    execute_externalities_with_wasm(0, protocol_fee, |wasm, code_hash| {
        let input_data = hex!("0222FF18");
        let extrinsic_wrapper_weight = 500_000_000;
        let instantiation_fee = 99999;

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

        // Get the balance of the Alice.
        let alice_balance = free(alice.acc());

        // Get the balance of the Bob.
        let bob_balance = free(bob.acc());

        // Create instance of contract.
        let result = create_contract_instance(bob.acc(), code_hash, instantiation_fee, false);
        // Verify the actual weight of the extrinsic.
        assert!(result.unwrap().actual_weight.unwrap() > extrinsic_wrapper_weight);

        // Verify whether the instantiation fee deducted properly or not.
        // Alice balance should increased by `instantiation_fee` and Bob balance should be decreased by the same amount.
        assert_eq!(bob_balance - free(bob.acc()), instantiation_fee + 100); // 100 for instantiation.
        assert_eq!(alice_balance + instantiation_fee, free(alice.acc()));

        // Generate the contract address.
        let addr_for = || {
            NonceBasedAddressDeterminer::<TestStorage>::contract_address_for(
                &code_hash,
                &input_data.to_vec(),
                &bob.acc(),
            )
        };
        let flipper_address_1 = addr_for();

        // Check whether the contract creation allowed or not with same constructor data.
        // It should be as contract creation is depend on the nonce of the account.
        assert_ok!(create_contract_instance(
            bob.acc(),
            code_hash,
            instantiation_fee,
            false
        ));

        // Verify that contract address is different.
        assert!(flipper_address_1 != addr_for());
    });
}

#[test]
fn allow_network_share_deduction() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    execute_externalities_with_wasm(25, protocol_fee, |wasm, code_hash| {
        let inst_fee = 5000;
        let fee_collector = account_from(5000);

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create template of SE:
        create_se_template(alice.acc(), alice.did, inst_fee, code_hash, wasm);

        // Get the balance of Alice.
        let alice_balance = free(alice.acc());
        // Get Network fee collector balance.
        let fee_collector_balance = free(fee_collector);

        // Create instance of contract.
        assert_ok!(create_contract_instance(
            bob.acc(),
            code_hash,
            inst_fee,
            false
        ));

        // Check the fee division.
        // 25 % of fee should be consumed by the network and 75% should be transferred to template owner.
        // 75% check
        assert_eq!(
            alice_balance.saturating_add(Perbill::from_percent(75) * inst_fee),
            free(alice.acc())
        );
        // 25% check
        assert_eq!(
            fee_collector_balance.saturating_add(Perbill::from_percent(25) * inst_fee),
            free(fee_collector)
        );
    });
}

#[test]
fn check_behavior_when_instantiation_fee_changes() {
    execute_externalities_with_wasm(30, <_>::default(), |wasm, code_hash| {
        let instantiation_fee = 5000;
        let fee_collector = account_from(5000);

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create template of SE.
        create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

        let new_instantiation_fee = 8000;

        // Change instantiation fee of the template
        // Should fail because provide hash doesn't exists
        assert_noop!(
            WrapperContracts::change_template_fees(
                alice.origin(),
                get_wrong_code_hash(),
                Some(new_instantiation_fee),
                None,
            ),
            WrapperContractsError::TemplateNotExists
        );

        // Should fail as sender is not the template owner
        assert_noop!(
            WrapperContracts::change_template_fees(
                bob.origin(),
                code_hash,
                Some(new_instantiation_fee),
                None,
            ),
            WrapperContractsError::UnAuthorizedOrigin
        );

        let old_template_fee = WrapperContracts::get_template_details(code_hash).instantiation_fee;

        // No change when None is passed.
        assert_ok!(WrapperContracts::change_template_fees(
            alice.origin(),
            code_hash,
            None,
            None,
        ));

        assert_eq!(
            WrapperContracts::get_template_details(code_hash).instantiation_fee,
            old_template_fee
        );

        // Should success fully change the instantiation fee
        assert_ok!(WrapperContracts::change_template_fees(
            alice.origin(),
            code_hash,
            Some(new_instantiation_fee),
            None,
        ));

        // Verify the storage changes
        assert_eq!(
            WrapperContracts::get_template_details(code_hash).instantiation_fee,
            new_instantiation_fee
        );

        // Get the balance of Alice.
        let alice_balance = free(alice.acc());
        // Get Network fee collector balance.
        let fee_collector_balance = free(fee_collector);

        // create instance of contract
        assert_ok!(create_contract_instance(
            bob.acc(),
            code_hash,
            new_instantiation_fee,
            false
        ));

        // Check the fee division.
        // 30 % of fee should be consumed by the network and 70% should be transferred to template owner.
        // 70% check
        assert_eq!(
            alice_balance.saturating_add(Perbill::from_percent(70) * new_instantiation_fee),
            free(alice.acc())
        );
        // 30% check
        assert_eq!(
            fee_collector_balance.saturating_add(Perbill::from_percent(30) * new_instantiation_fee),
            free(fee_collector)
        );
    });
}

#[test]
fn check_freeze_unfreeze_functionality() {
    execute_externalities_with_wasm(30, <_>::default(), |wasm, code_hash| {
        let instantiation_fee = 5000;

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create template of SE.
        create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

        let freeze = || WrapperContracts::freeze_instantiation(alice.origin(), code_hash);
        let unfreeze = || WrapperContracts::unfreeze_instantiation(alice.origin(), code_hash);
        let frozen = || WrapperContracts::get_template_details(code_hash).frozen;

        // Check whether freeze functionality is working or not
        // successfully freeze the instantiation of the SE template.
        assert_ok!(freeze());
        assert!(frozen());

        // Should fail when trying to freeze the template again
        assert_noop!(freeze(), WrapperContractsError::InstantiationAlreadyFrozen);

        // Instantiation should fail.
        let create = |fee, fail| create_contract_instance(bob.acc(), code_hash, fee, fail);
        assert_noop!(
            create(instantiation_fee, true),
            WrapperContractsError::InstantiationIsNotAllowed
        );

        // Check unfreeze functionality.

        // Successfully unfreeze the instantiation of the SE template.
        assert_ok!(unfreeze());
        assert!(!frozen());

        // Should fail when trying to unfreeze the template again
        assert_noop!(
            unfreeze(),
            WrapperContractsError::InstantiationAlreadyUnFrozen
        );

        // Instantiation should fail if we max_fee is less than the instantiation fee.
        assert_noop!(create(500, true), WrapperContractsError::InsufficientMaxFee);

        // Instantiation should passed
        assert_ok!(create(instantiation_fee, false));
    });
}

#[test]
fn validate_transfer_template_ownership_functionality() {
    // Build wasm and get code_hash
    let (code_hash, wasm) = flipper();

    ExtBuilder::default()
        .network_fee_share(Perbill::from_percent(30))
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_contracts_put_code(true)
        .build()
        .execute_with(|| {
            let instantiation_fee = 5000;

            let alice = User::new(AccountKeyring::Alice);

            // Create Bob account & the identity for her.
            let bob = AccountKeyring::Bob.public();
            let bob_uid = InvestorUid::from("bob_take_1");
            let (_, bob_did) = make_account_with_uid(bob, bob_uid).unwrap();

            // Create template of se
            create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

            // Call the transfer ownership functionality
            // Should fail because provided identityId doesn't has the CDD
            assert_noop!(
                WrapperContracts::transfer_template_ownership(
                    alice.origin(),
                    code_hash,
                    IdentityId::from(2)
                ),
                WrapperContractsError::NewOwnerIsNotCDD
            );

            // Not a valid sender.
            assert_noop!(
                WrapperContracts::transfer_template_ownership(
                    Origin::signed(account_from(45)),
                    code_hash,
                    bob_did
                ),
                PermissionError::UnauthorizedCaller
            );

            // Successfully transfer ownership to the other DID.
            assert_ok!(WrapperContracts::transfer_template_ownership(
                alice.origin(),
                code_hash,
                bob_did
            ));

            assert_eq!(
                WrapperContracts::get_template_details(code_hash).owner,
                bob_did
            );
        });
}

#[test]
fn check_transaction_rollback_functionality_for_put_code() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 900000000)]);

    execute_externalities_with_wasm(30, protocol_fee, |wasm, code_hash| {
        let instantiation_fee = 5000;
        let alice = User::new(AccountKeyring::Alice);

        // Set payer in context
        TestStorage::set_payer_context(Some(alice.acc()));

        // Create smart extension metadata
        let se_meta_data = TemplateMetadata {
            url: None,
            se_type: SmartExtensionType::TransferManager,
            usage_fee: 0,
            description: "This is a transfer manager type contract".into(),
            version: 5000,
        };

        // Execute `put_code`
        assert_noop!(
            WrapperContracts::put_code(
                alice.origin(),
                se_meta_data.clone(),
                instantiation_fee,
                wasm
            ),
            ProtocolFeeError::InsufficientAccountBalance
        );

        // Verify that storage doesn't change.
        assert!(!MetadataOfTemplate::<TestStorage>::contains_key(code_hash));
        assert!(<pallet_contracts::PristineCode<TestStorage>>::get(code_hash).is_none())
    });
}

#[test]
fn check_transaction_rollback_functionality_for_instantiation() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    execute_externalities_with_wasm(30, protocol_fee, |wasm, code_hash| {
        let input_data = hex!("0222FF18");
        let instantiation_fee = 10000000000;
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create template of se
        create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

        // create instance of contract
        assert_noop!(
            create_contract_instance(bob.acc(), code_hash, instantiation_fee, true),
            ProtocolFeeError::InsufficientAccountBalance
        );

        // Generate the contract address.
        let flipper_address_1 = NonceBasedAddressDeterminer::<TestStorage>::contract_address_for(
            &code_hash,
            &input_data.to_vec(),
            &bob.acc(),
        );

        assert!(!ContractInfoOf::<TestStorage>::contains_key(
            flipper_address_1
        ));
    });
}

#[test]
fn check_meta_url_functionality() {
    let protocol_fee = MockProtocolBaseFees(vec![(ProtocolOp::ContractsPutCode, 500)]);

    execute_externalities_with_wasm(30, protocol_fee, |wasm, code_hash| {
        let instantiation_fee = 10000000000;
        let alice = User::new(AccountKeyring::Alice);

        // Create template of SE.
        create_se_template(alice.acc(), alice.did, instantiation_fee, code_hash, wasm);

        // Change the meta url.
        assert_ok!(WrapperContracts::change_template_meta_url(
            alice.origin(),
            code_hash,
            Some("http://www.google.com".into())
        ));
    });
}

#[test]
fn check_put_code_flag() {
    let user = AccountKeyring::Charlie.public();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[user])
        .build()
        .execute_with(|| check_put_code_flag_ext(user))
}

fn check_put_code_flag_ext(user: Public) {
    let (_, wasm) = flipper();
    let put_code = |acc: Public| -> DispatchResult {
        WrapperContracts::put_code(
            Origin::signed(acc),
            TemplateMetadata::default(),
            0u128,
            wasm.clone(),
        )
    };

    // Flag is disable, so `put_code` should fail.
    assert_noop!(put_code(user), WrapperContractsError::PutCodeIsNotAllowed);

    // Non GC member cannot update the flag.
    assert_noop!(
        WrapperContracts::set_put_code_flag(Origin::signed(user), true),
        DispatchError::BadOrigin
    );

    // Enable and check that anyone now can put code.
    let root = Origin::from(RawOrigin::Root);
    assert_ok!(WrapperContracts::set_put_code_flag(root, true));
    assert_ok!(put_code(user));
}
