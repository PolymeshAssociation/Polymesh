// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use sp_runtime::traits::Zero;

use rand_chacha::ChaCha20Rng as StdRng;
use rand_core::SeedableRng;

use mercat::{
    account::AccountCreator,
    asset::AssetIssuer,
    confidential_identity_core::{
        asset_proofs::ElgamalSecretKey, curve25519_dalek::scalar::Scalar,
    },
    transaction::CtxSender,
    Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptedAmount, EncryptionKeys,
    PubAccount, PubAccountTx, SecAccount, TransferTransactionSender,
};

use polymesh_common_utilities::{
    benchs::{user, AccountIdOf, User},
    traits::TestUtilsFn,
};
use polymesh_primitives::{
    asset::{AssetName, AssetType},
    Ticker,
};

use crate::*;

pub trait ConfigT<T: frame_system::Config>: Config + TestUtilsFn<AccountIdOf<T>> {}

pub(crate) const SEED: u32 = 42;

fn create_confidential_token<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    user: &User<T>,
    token_name: &[u8],
    ticker: Ticker,
) {
    assert_ok!(Module::<T>::create_confidential_asset(
        user.origin().into(),
        AssetName(token_name.into()),
        ticker,
        AssetType::default(),
    ));
}

#[derive(Clone, Debug)]
pub struct MercatUser<T: Config + TestUtilsFn<AccountIdOf<T>>> {
    pub user: User<T>,
    pub sec: SecAccount,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> MercatUser<T> {
    /// Creates a mercat user.
    pub fn new(name: &str, rng: &mut StdRng) -> Self {
        let user = user::<T>(name, SEED);
        Self::new_from_user(user, rng)
    }

    /// Creates a mercat user.
    pub fn new_from_user(user: User<T>, rng: &mut StdRng) -> Self {
        // These are the encryptions keys used by MERCAT and are different from the signing keys
        // that Polymesh uses.
        let elg_secret = ElgamalSecretKey::new(Scalar::random(rng));
        let elg_pub = elg_secret.get_public_key();

        Self {
            user,
            sec: SecAccount {
                enc_keys: EncryptionKeys {
                    public: elg_pub.into(),
                    secret: elg_secret.into(),
                },
            },
        }
    }

    pub fn pub_account(&self) -> PubAccount {
        PubAccount {
            owner_enc_pub_key: self.sec.enc_keys.public,
        }
    }

    pub fn pub_key(&self) -> EncryptionPubKey {
        self.sec.enc_keys.public
    }

    pub fn mercat(&self) -> MercatAccount {
        MercatAccount {
            pub_key: self.sec.enc_keys.public.into(),
        }
    }

    pub fn did(&self) -> IdentityId {
        self.user.did()
    }

    pub fn origin(&self) -> frame_system::RawOrigin<T::AccountId> {
        self.user.origin()
    }

    /// Create account initial proof.
    pub fn account_tx(&self, rng: &mut StdRng) -> PubAccountTx {
        AccountCreator.create(&self.sec, rng).unwrap()
    }

    /// Create asset mint proof.
    pub fn mint_tx(&self, amount: u32, rng: &mut StdRng) -> InitializedAssetTxWrapper {
        let issuer_account = Account {
            secret: self.sec.clone(),
            public: self.pub_account(),
        };

        let initialized_asset_tx = AssetIssuer
            .initialize_asset_transaction(&issuer_account, &[], amount, rng)
            .unwrap();
        InitializedAssetTxWrapper::from(initialized_asset_tx)
    }

    /// Initialize a new mercat account on-chain for `ticker`.
    pub fn init_account(&self, ticker: Ticker, rng: &mut StdRng) {
        let mercat_account_tx = self.account_tx(rng);
        assert_ok!(Module::<T>::validate_mercat_account(
            self.origin().into(),
            ticker,
            PubAccountTxWrapper::from(mercat_account_tx.clone())
        ));
    }

    pub fn mercat_enc_balance(&self, ticker: Ticker) -> EncryptedAmount {
        *Module::<T>::mercat_account_balance(self.mercat(), ticker)
    }

    pub fn ensure_mercat_balance(&self, ticker: Ticker, balance: u32) {
        let enc_balance = self.mercat_enc_balance(ticker);
        self.sec
            .enc_keys
            .secret
            .verify(&enc_balance, &balance.into())
            .expect("mercat balance")
    }

    pub fn add_mediator(&self) {
        assert_ok!(Module::<T>::add_mediator_mercat_account(
            self.origin().into(),
            EncryptionPubKeyWrapper::from(self.pub_key()),
        ));
    }
}

/// Create issuer's mercat account, create asset and mint.
pub fn create_account_and_mint_token<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    name: &str,
    total_supply: u128,
    token_name: &[u8],
    rng: &mut StdRng,
) -> (Ticker, MercatUser<T>) {
    let owner = MercatUser::new(name, rng);
    let token = ConfidentialAssetDetails {
        total_supply,
        owner_did: owner.did(),
        asset_type: AssetType::default(),
    };
    let ticker = Ticker::from_slice_truncated(token_name);

    create_confidential_token(&owner.user, token_name, ticker);

    // In the initial call, the total_supply must be zero.
    assert_eq!(
        Module::<T>::confidential_asset_details(ticker).total_supply,
        Zero::zero()
    );

    // ---------------- prepare for minting the asset

    owner.init_account(ticker, rng);

    // ------------- Computations that will happen in owner's Wallet ----------
    let amount: u32 = token.total_supply.try_into().unwrap(); // mercat amounts are 32 bit integers.
    let mint_tx = owner.mint_tx(amount, rng);

    // Wallet submits the transaction to the chain for verification.
    assert_ok!(Module::<T>::mint_confidential_asset(
        owner.origin().into(),
        ticker,
        amount.into(), // convert to u128
        mint_tx,
    ));

    // ------------------------- Ensuring that the asset details are set correctly

    // A correct entry is added.
    assert_eq!(
        Module::<T>::confidential_asset_details(ticker).owner_did,
        token.owner_did
    );

    // -------------------------- Ensure the encrypted balance matches the minted amount.
    owner.ensure_mercat_balance(ticker, amount);

    (ticker, owner)
}

#[derive(Clone)]
pub struct TransactionState<T: Config + TestUtilsFn<AccountIdOf<T>>> {
    pub ticker: Ticker,
    pub amount: u32,
    pub issuer: MercatUser<T>,
    pub investor: MercatUser<T>,
    pub mediator: MercatUser<T>,
    pub venue_id: VenueId,
    pub legs: Vec<TransactionLeg>,
    pub id: TransactionId,
}

impl<T: Config + TestUtilsFn<AccountIdOf<T>>> TransactionState<T> {
    /// Create 3 mercat accounts (issuer, investor, mediator), create asset, mint.
    pub fn new(amount: u32, rng: &mut StdRng) -> Self {
        // Setup confidential asset.
        let total_supply = 10_000_000;
        let (ticker, issuer) =
            create_account_and_mint_token::<T>("issuer", total_supply, b"A", rng);

        // Setup mediator.
        let mediator = MercatUser::<T>::new("mediator", rng);
        mediator.add_mediator();

        // Setup venue.
        // TODO:
        let venue_id = VenueId(0);

        // Setup investor.
        let investor = MercatUser::<T>::new("investor", rng);
        investor.init_account(ticker, rng);

        let legs = vec![TransactionLeg {
            ticker,
            sender: issuer.mercat(),
            receiver: investor.mercat(),
            mediator: mediator.did(),
        }];
        Self {
            ticker,
            amount,
            issuer,
            investor,
            mediator,
            venue_id,
            legs,
            id: Default::default(),
        }
    }

    pub fn add_transaction(&mut self) {
        self.id = Module::<T>::transaction_counter();
        assert_ok!(Module::<T>::add_transaction(
            self.issuer.origin().into(),
            self.venue_id,
            self.legs.clone(),
        ));
    }

    pub fn sender_proof(&self, rng: &mut StdRng) -> TransactionLegProofs {
        let issuer_account = Account {
            secret: self.issuer.sec.clone(),
            public: self.issuer.pub_account(),
        };
        let investor_pub_account = self.investor.pub_account();
        let issuer_enc_balance = self.issuer.mercat_enc_balance(self.ticker);
        let sender_tx = CtxSender
            .create_transaction(
                &issuer_account,
                &issuer_enc_balance,
                &investor_pub_account,
                &self.mediator.pub_key(),
                &[],
                self.amount,
                rng,
            )
            .unwrap();
        TransactionLegProofs::new_sender(sender_tx)
    }

    pub fn receiver_proof(&self) -> TransactionLegProofs {
        TransactionLegProofs::new_receiver(FinalizedTransferTx {})
    }

    pub fn mediator_proof(&self) -> TransactionLegProofs {
        TransactionLegProofs::new_mediator(JustifiedTransferTx {})
    }

    pub fn sender_affirm(&self, leg_id: TransactionLegId, rng: &mut StdRng) {
        let proof = self.sender_proof(rng);
        assert_ok!(Module::<T>::affirm_transaction(
            self.issuer.origin().into(),
            self.id,
            leg_id,
            proof
        ));
    }

    pub fn receiver_affirm(&self, leg_id: TransactionLegId) {
        let proof = self.receiver_proof();
        assert_ok!(Module::<T>::affirm_transaction(
            self.investor.origin().into(),
            self.id,
            leg_id,
            proof,
        ));
    }

    pub fn mediator_affirm(&self, leg_id: TransactionLegId) {
        let proof = self.mediator_proof();
        assert_ok!(Module::<T>::affirm_transaction(
            self.investor.origin().into(),
            self.id,
            leg_id,
            proof,
        ));
    }

    pub fn execute(&self) {
        assert_ok!(Module::<T>::execute_transaction(
            self.issuer.origin().into(),
            self.id,
            self.legs.len() as u32,
        ));
    }
}

benchmarks! {
    where_clause { where T: Config, T: TestUtilsFn<AccountIdOf<T>> }

    validate_mercat_account {
        let mut rng = StdRng::from_seed([10u8; 32]);
        let ticker = Ticker::from_slice_truncated(b"A".as_ref());
        let user = MercatUser::<T>::new("user", &mut rng);
        let mercat_account_tx = user.account_tx(&mut rng);
        let account_tx = PubAccountTxWrapper::from(mercat_account_tx.clone());

    }: _(user.origin(), ticker, account_tx)

    add_mediator_mercat_account {
        let mut rng = StdRng::from_seed([10u8; 32]);
        let mediator = MercatUser::<T>::new("mediator", &mut rng);
        let pub_key = EncryptionPubKeyWrapper::from(mediator.pub_key());
    }: _(mediator.origin(), pub_key)

    create_confidential_asset {
        let ticker = Ticker::from_slice_truncated(b"A".as_ref());
        let issuer = user::<T>("issuer", SEED);
    }: _(issuer.origin(), AssetName(b"Name".to_vec()), ticker, AssetType::default())

    mint_confidential_asset {
        let mut rng = StdRng::from_seed([10u8; 32]);
        let issuer = MercatUser::<T>::new("issuer", &mut rng);
        let ticker = Ticker::from_slice_truncated(b"A".as_ref());
        create_confidential_token(
            &issuer.user,
            b"Name".as_slice(),
            ticker,
        );
        issuer.init_account(ticker, &mut rng);

        let amount = 10_000_000u32; // mercat amounts are 32 bit integers.
        let mint_tx = issuer.mint_tx(amount, &mut rng);
    }: _(issuer.origin(), ticker, amount.into(), mint_tx)

    add_transaction {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let tx = TransactionState::<T>::new(10_000, &mut rng);

    }: _(tx.issuer.origin(), tx.venue_id, tx.legs)

    sender_affirm_transaction {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let mut tx = TransactionState::<T>::new(10_000, &mut rng);
        tx.add_transaction();
        let leg_id = TransactionLegId(0);

        let proof = tx.sender_proof(&mut rng);
    }: affirm_transaction(tx.issuer.origin(), tx.id, leg_id, proof)

    receiver_affirm_transaction {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let mut tx = TransactionState::<T>::new(10_000, &mut rng);
        tx.add_transaction();
        let leg_id = TransactionLegId(0);
        tx.sender_affirm(leg_id, &mut rng);

        let proof = tx.receiver_proof();
    }: affirm_transaction(tx.investor.origin(), tx.id, leg_id, proof)

    mediator_affirm_transaction {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let mut tx = TransactionState::<T>::new(10_000, &mut rng);
        tx.add_transaction();
        let leg_id = TransactionLegId(0);
        tx.sender_affirm(leg_id, &mut rng);
        tx.receiver_affirm(leg_id);

        let proof = tx.mediator_proof();
    }: affirm_transaction(tx.mediator.origin(), tx.id, leg_id, proof)

    execute_transaction {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let mut tx = TransactionState::<T>::new(10_000, &mut rng);
        tx.add_transaction();
        let leg_id = TransactionLegId(0);
        tx.sender_affirm(leg_id, &mut rng);
        tx.receiver_affirm(leg_id);
        tx.mediator_affirm(leg_id);
    }: _(tx.issuer.origin(), tx.id, 1)

    reset_ordering_state {
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setup for transaction.
        let mut tx = TransactionState::<T>::new(10_000, &mut rng);
        tx.add_transaction();
        let leg_id = TransactionLegId(0);
        tx.sender_affirm(leg_id, &mut rng);
        tx.receiver_affirm(leg_id);
        tx.mediator_affirm(leg_id);
        tx.execute();
    }: _(tx.issuer.origin(), tx.issuer.mercat(), tx.ticker)
}
