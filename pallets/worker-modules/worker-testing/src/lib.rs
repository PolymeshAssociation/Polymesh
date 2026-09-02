// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh

//! # Confidential assets Pallet
//!
//! The Confidential Assets pallet provides sender, receiver, asset and value confidentiality.
//!
//! ## Overview
//!
//! These pallets call out to the [Polymesh DART library](https://github.com/PolymeshAssociation/polymesh-dart)
//! which implements the ZK-proofs for DART.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchError;
use frame_support::{dispatch::DispatchResult, weights::Weight};
use frame_system::pallet_prelude::*;

use polymesh_worker_common::{
    BackendKind, Protocol, WorkRequestConfig, WorkerSessionConfig, WorkerSessionId,
};
use polymesh_worker_extension::native_polymesh_worker;
use polymesh_worker_protocol_testing::{
    TestWorkRequest, TestWorkResponse, VerifyVersionRequest, PROTOCOL as TEST_PROTOCOL,
};

pub mod weights;

pub trait WeightInfo {
    fn test_version() -> Weight;
    fn submit_work_request() -> Weight;
    fn set_protocol_version() -> Weight;
    fn set_enable_work_session() -> Weight;
    fn on_init() -> Weight;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use polymesh_worker_common::ProtocolError;
    use polymesh_worker_protocol_testing::{TestWorkRequest, TestWorkResponse};

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Confidential asset pallet weights.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TestingProtocolTask {
            request: TestWorkRequest,
            result: Result<TestWorkResponse, ProtocolError>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The worker session is not initialized.
        NoSession,
    }

    /// The WorkerSessionId for the current block.
    #[pallet::storage]
    pub(crate) type CurrentWorkerSessionId<T: Config> =
        StorageValue<_, WorkerSessionId, OptionQuery>;

    /// Enable work session per block. This is used for testing purpose only.
    #[pallet::storage]
    pub(crate) type EnableWorkSession<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// The current testing protocol version.
    #[pallet::storage]
    pub(crate) type CurrentProtocolVersion<T: Config> = StorageValue<_, Protocol, OptionQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            CurrentProtocolVersion::<T>::put(TEST_PROTOCOL);
            EnableWorkSession::<T>::put(false);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Self::init_block()
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            Self::finalize_block();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Test the protocol version in the `Testing` protocol module.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::test_version())]
        pub fn test_version(origin: OriginFor<T>, protocol: Protocol) -> DispatchResult {
            ensure_signed(origin)?;

            // Test verify protocol version work request.
            let request = TestWorkRequest::VerifyVersion(VerifyVersionRequest { protocol });
            Self::session_submit_work_request(request)?;

            Ok(())
        }

        /// Set what version of the testing protocol.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::set_protocol_version())]
        pub fn set_protocol_version(origin: OriginFor<T>, protocol: Protocol) -> DispatchResult {
            ensure_root(origin)?;
            CurrentProtocolVersion::<T>::put(protocol);
            Ok(())
        }

        /// Set whether to enable work session per block. This is used for testing purpose only.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_enable_work_session())]
        pub fn set_enable_work_session(origin: OriginFor<T>, enable: bool) -> DispatchResult {
            ensure_root(origin)?;
            EnableWorkSession::<T>::put(enable);
            Ok(())
        }

        /// Submit a work request to the worker session.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_work_request())]
        pub fn submit_work_request(
            origin: OriginFor<T>,
            request: TestWorkRequest,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            Self::session_submit_work_request(request)?;
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn init_block() -> Weight {
        if EnableWorkSession::<T>::get() {
            Self::start_session();
        }
        <T as Config>::WeightInfo::on_init()
    }

    pub fn finalize_block() {
        Self::end_session();
    }

    pub fn start_session() {
        // Start worker session.
        let config = WorkerSessionConfig {
            work: WorkRequestConfig {
                use_cache: true,
                use_thread_pool: false,
            },
            init_module: true,

            backends: BackendKind::all_mask(),
        }
        .to_flags_and_backends();
        let protocol = CurrentProtocolVersion::<T>::get().unwrap_or(TEST_PROTOCOL);
        let session_id = native_polymesh_worker::start_session(config, protocol.to_number());

        CurrentWorkerSessionId::<T>::put(session_id);
    }

    pub fn session_submit_work_request(
        request: TestWorkRequest,
    ) -> Result<Option<TestWorkResponse>, DispatchError> {
        let session_id = CurrentWorkerSessionId::<T>::get().ok_or(Error::<T>::NoSession)?;
        let result = request.clone().session_execute_and_wait(session_id);

        Self::deposit_event(Event::TestingProtocolTask {
            request,
            result: result.clone(),
        });

        Ok(result.ok())
    }

    pub fn end_session() {
        // Close the batch.
        if let Some(session_id) = CurrentWorkerSessionId::<T>::take() {
            native_polymesh_worker::end_session(session_id);
        }
    }
}
