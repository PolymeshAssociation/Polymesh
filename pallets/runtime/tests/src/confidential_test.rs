use super::{
    storage::{TestStorage, User},
    ExtBuilder,
};

use confidential_identity_v1::{compute_cdd_id, compute_scope_id};
use pallet_asset as asset;
use pallet_compliance_manager as compliance_manager;
use pallet_confidential as confidential;
use pallet_identity as identity;
use polymesh_common_utilities::constants::ERC1400_TRANSFER_SUCCESS;
use polymesh_primitives::{
    asset::{AssetName, AssetType, SecurityToken},
    investor_zkproof_data::v1::InvestorZKProofData,
    AssetIdentifier, Claim, IdentityId, InvestorUid, PortfolioId, Scope, Ticker,
};

use core::convert::TryFrom;
use frame_support::{assert_err, assert_ok};
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type IdentityError = identity::Error<TestStorage>;
type Asset = asset::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type Confidential = confidential::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;

#[test]
fn range_proof() {
    ExtBuilder::default().build().execute_with(range_proof_we);
}

fn range_proof_we() {
    let alice = User::new(AccountKeyring::Alice);
    let prover = User::new(AccountKeyring::Bob);
    let verifier = User::new(AccountKeyring::Charlie);

    // 1. Alice creates her security token.
    let token_name = "ALI_ST".as_bytes().to_owned();
    let token = SecurityToken {
        owner_did: alice.did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![AssetIdentifier::isin(*b"US0378331005").unwrap()];
    let ticker = Ticker::try_from(token_name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        alice.origin(),
        AssetName(token_name.clone()),
        ticker,
        true,
        token.asset_type.clone(),
        identifiers.clone(),
        None,
        true,
    ));
    assert_ok!(Asset::issue(alice.origin(), ticker, token.total_supply));

    // 2. X add a range proof
    let secret_value = 42;
    assert_ok!(Confidential::add_range_proof(
        prover.origin(),
        alice.did,
        ticker.clone(),
        secret_value,
    ));

    assert_ok!(Confidential::add_verify_range_proof(
        verifier.origin(),
        alice.did,
        prover.did,
        ticker.clone()
    ));

    assert_eq!(
        Confidential::range_proof_verification((alice.did, ticker), verifier.did),
        true
    );
}

#[test]
fn scope_claims() {
    ExtBuilder::default()
        .monied(true)
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(scope_claims_we);
}

fn scope_claims_we() {
    let alice = User::new(AccountKeyring::Alice);
    let investor = InvestorUid::from("inv_1");
    let inv_acc_1 = User::new_with_uid(AccountKeyring::Bob, investor);
    let inv_acc_2 = User::new_with_uid(AccountKeyring::Charlie, investor);
    let inv_acc_3 = User::new(AccountKeyring::Dave);

    // 1. Alice creates her ST and set up its compliance requirements.
    let st_name = "ALI_ST".as_bytes().to_owned();
    let st = SecurityToken {
        owner_did: alice.did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![AssetIdentifier::isin(*b"US0378331005").unwrap()];
    let st_id = Ticker::try_from(st_name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        alice.origin(),
        AssetName(st_name.clone()),
        st_id,
        true,
        st.asset_type.clone(),
        identifiers.clone(),
        None,
        false,
    ));
    assert_ok!(Asset::issue(alice.origin(), st_id, st.total_supply));

    // 2. Alice defines the asset compliance requirements.
    let st_scope = Scope::Identity(IdentityId::try_from(st_id.as_slice()).unwrap());
    assert_ok!(ComplianceManager::add_compliance_requirement(
        alice.origin(),
        st_id,
        vec![],
        vec![]
    ));

    // 2. Investor adds its Confidential Scope claims.
    let scope_claim = InvestorZKProofData::make_scope_claim(&st_id.as_slice(), &investor);
    let scope_id = compute_scope_id(&scope_claim).compress().to_bytes().into();

    let inv_1_proof = InvestorZKProofData::new(&inv_acc_1.did, &investor, &st_id);
    let cdd_claim_1 = InvestorZKProofData::make_cdd_claim(&inv_acc_1.did, &investor);
    let cdd_id_1 = compute_cdd_id(&cdd_claim_1).compress().to_bytes().into();

    let conf_scope_claim_error = Claim::InvestorUniqueness(st_scope, scope_id, cdd_id_1);
    let conf_scope_claim_1 = Claim::InvestorUniqueness(st_id.into(), scope_id, cdd_id_1);

    assert_err!(
        Identity::add_investor_uniqueness_claim(
            inv_acc_1.origin(),
            inv_acc_1.did,
            conf_scope_claim_error.clone(),
            inv_1_proof.clone(),
            None
        ),
        IdentityError::InvalidScopeClaim
    );

    assert_ok!(Identity::add_investor_uniqueness_claim(
        inv_acc_1.origin(),
        inv_acc_1.did,
        conf_scope_claim_1.clone(),
        inv_1_proof.clone(),
        None
    ),);

    let inv_2_proof = InvestorZKProofData::new(&inv_acc_2.did, &investor, &st_id);
    let cdd_claim_2 = InvestorZKProofData::make_cdd_claim(&inv_acc_2.did, &investor);
    let cdd_id_2 = compute_cdd_id(&cdd_claim_2).compress().to_bytes().into();

    let conf_scope_claim_2 = Claim::InvestorUniqueness(st_id.into(), scope_id, cdd_id_2);
    assert_ok!(Identity::add_investor_uniqueness_claim(
        inv_acc_2.origin(),
        inv_acc_2.did,
        conf_scope_claim_2,
        inv_2_proof,
        None
    ));

    // 3. Transfer some tokens to Inv. 1 and 2.
    assert_eq!(Asset::balance_of(st_id, inv_acc_1.did), 0);
    assert_ok!(Asset::unsafe_transfer(
        PortfolioId::default_portfolio(alice.did),
        PortfolioId::default_portfolio(inv_acc_1.did),
        &st_id,
        10
    ));
    assert_eq!(Asset::balance_of(st_id, inv_acc_1.did), 10);

    assert_eq!(Asset::balance_of(st_id, inv_acc_2.did), 0);
    assert_ok!(Asset::unsafe_transfer(
        PortfolioId::default_portfolio(alice.did),
        PortfolioId::default_portfolio(inv_acc_2.did),
        &st_id,
        20
    ));
    assert_eq!(Asset::balance_of(st_id, inv_acc_2.did), 20);

    // 4. ERROR: Investor 2 cannot add a claim of the real investor.
    assert_err!(
        Identity::add_investor_uniqueness_claim(
            inv_acc_3.origin(),
            inv_acc_3.did,
            conf_scope_claim_1,
            inv_1_proof.clone(),
            None
        ),
        IdentityError::InvalidCDDId
    );

    // 5. ERROR: Replace the scope
    let st_2_name = "ALI2_ST".as_bytes().to_owned();
    let st_2 = SecurityToken {
        owner_did: alice.did,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let identifiers = vec![AssetIdentifier::isin(*b"US0378331005").unwrap()];
    let st2_id = Ticker::try_from(st_2_name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        alice.origin(),
        AssetName(st_2_name.clone()),
        st2_id,
        true,
        st_2.asset_type.clone(),
        identifiers.clone(),
        None,
        false,
    ));
    assert_ok!(Asset::issue(alice.origin(), st2_id, st_2.total_supply));

    let corrupted_scope_claim =
        InvestorZKProofData::make_scope_claim(&st2_id.as_slice(), &investor);
    let corrupted_scope_id = compute_scope_id(&corrupted_scope_claim)
        .compress()
        .to_bytes()
        .into();

    let conf_scope_claim_3 = Claim::InvestorUniqueness(st2_id.into(), corrupted_scope_id, cdd_id_1);
    assert_err!(
        Identity::add_investor_uniqueness_claim(
            inv_acc_1.origin(),
            inv_acc_1.did,
            conf_scope_claim_3.clone(),
            inv_1_proof.clone(),
            None
        ),
        IdentityError::InvalidScopeClaim
    );

    assert_ne!(
        Asset::_is_valid_transfer(
            &st2_id,
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::default_portfolio(inv_acc_1.did),
            10
        ),
        Ok(ERC1400_TRANSFER_SUCCESS)
    );
}
