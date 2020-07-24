#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}
pub mod asset {
    pub use node_rpc_runtime_api::asset::{AssetApi as AssetRuntimeApi, CanTransferResult};
    use polymesh_primitives::{IdentityId, Ticker};
    use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
    use jsonrpc_derive::rpc;
    use codec::Codec;
    use sp_api::{ApiRef, ProvideRuntimeApi};
    use sp_blockchain::HeaderBackend;
    use sp_runtime::{generic::BlockId, traits::Block as BlockT};
    use std::sync::Arc;
    mod rpc_impl_AssetApi {
        use jsonrpc_core as _jsonrpc_core;
        use super::*;
        /// The generated client module.
        pub mod gen_client {
            use jsonrpc_core_client as _jsonrpc_core_client;
            use super::*;
            use _jsonrpc_core::{
                Call, Error, ErrorCode, Id, MethodCall, Params, Request, Response, Version,
            };
            use _jsonrpc_core::futures::prelude::*;
            use _jsonrpc_core::futures::sync::{mpsc, oneshot};
            use _jsonrpc_core::serde_json::{self, Value};
            use _jsonrpc_core_client::{
                RpcChannel, RpcError, RpcFuture, TypedClient, TypedSubscriptionStream,
            };
            /// The Client.
            pub struct Client<BlockHash, AccountId, T> {
                inner: TypedClient,
                _0: std::marker::PhantomData<BlockHash>,
                _1: std::marker::PhantomData<AccountId>,
                _2: std::marker::PhantomData<T>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl<
                    BlockHash: ::core::clone::Clone,
                    AccountId: ::core::clone::Clone,
                    T: ::core::clone::Clone,
                > ::core::clone::Clone for Client<BlockHash, AccountId, T>
            {
                #[inline]
                fn clone(&self) -> Client<BlockHash, AccountId, T> {
                    match *self {
                        Client {
                            inner: ref __self_0_0,
                            _0: ref __self_0_1,
                            _1: ref __self_0_2,
                            _2: ref __self_0_3,
                        } => Client {
                            inner: ::core::clone::Clone::clone(&(*__self_0_0)),
                            _0: ::core::clone::Clone::clone(&(*__self_0_1)),
                            _1: ::core::clone::Clone::clone(&(*__self_0_2)),
                            _2: ::core::clone::Clone::clone(&(*__self_0_3)),
                        },
                    }
                }
            }
            impl<BlockHash, AccountId, T> Client<BlockHash, AccountId, T>
            where
                BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                AccountId: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                T: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
            {
                /// Creates a new `Client`.
                pub fn new(sender: RpcChannel) -> Self {
                    Client {
                        inner: sender.into(),
                        _0: std::marker::PhantomData,
                        _1: std::marker::PhantomData,
                        _2: std::marker::PhantomData,
                    }
                }
                pub fn can_transfer(
                    &self,
                    sender: AccountId,
                    ticker: Ticker,
                    from_did: Option<IdentityId>,
                    to_did: Option<IdentityId>,
                    value: T,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = CanTransferResult, Error = RpcError> {
                    let args = (sender, ticker, from_did, to_did, value, at);
                    self.inner
                        .call_method("asset_canTransfer", "CanTransferResult", args)
                }
            }
            impl<BlockHash, AccountId, T> From<RpcChannel> for Client<BlockHash, AccountId, T>
            where
                BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                AccountId: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                T: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
            {
                fn from(channel: RpcChannel) -> Self {
                    Client::new(channel.into())
                }
            }
        }
        /// The generated server module.
        pub mod gen_server {
            use self::_jsonrpc_core::futures as _futures;
            use super::*;
            pub trait AssetApi<BlockHash, AccountId, T>: Sized + Send + Sync + 'static {
                fn can_transfer(
                    &self,
                    sender: AccountId,
                    ticker: Ticker,
                    from_did: Option<IdentityId>,
                    to_did: Option<IdentityId>,
                    value: T,
                    at: Option<BlockHash>,
                ) -> Result<CanTransferResult>;
                /// Create an `IoDelegate`, wiring rpc calls to the trait methods.
                fn to_delegate<M: _jsonrpc_core::Metadata>(
                    self,
                ) -> _jsonrpc_core::IoDelegate<Self, M>
                where
                    BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
                    AccountId: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
                    T: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
                {
                    let mut del = _jsonrpc_core::IoDelegate::new(self.into());
                    del.add_method("asset_canTransfer", move |base, params| {
                        let method = &(Self::can_transfer
                            as fn(
                                &Self,
                                AccountId,
                                Ticker,
                                Option<IdentityId>,
                                Option<IdentityId>,
                                T,
                                Option<BlockHash>,
                            ) -> Result<CanTransferResult>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 5usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&5usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                5usize => {
                                    params
                                        .parse::<(
                                            AccountId,
                                            Ticker,
                                            Option<IdentityId>,
                                            Option<IdentityId>,
                                            T,
                                        )>()
                                        .map(|(a, b, c, d, e)| (a, b, c, d, e, None))
                                        .map_err(Into::into)
                                }
                                6usize => params
                                    .parse::<(
                                        AccountId,
                                        Ticker,
                                        Option<IdentityId>,
                                        Option<IdentityId>,
                                        T,
                                        Option<BlockHash>,
                                    )>()
                                    .map(|(a, b, c, d, e, f)| (a, b, c, d, e, f))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&5usize, &6usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b, c, d, e, f)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b, c, d, e, f)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del
                }
            }
        }
    }
    pub use self::rpc_impl_AssetApi::gen_client;
    pub use self::rpc_impl_AssetApi::gen_server::AssetApi;
    /// An implementation of asset specific RPC methods.
    pub struct Asset<T, U> {
        client: Arc<T>,
        _marker: std::marker::PhantomData<U>,
    }
    impl<T, U> Asset<T, U> {
        /// Create new `Asset` with the given reference to the client.
        pub fn new(client: Arc<T>) -> Self {
            Self {
                client,
                _marker: Default::default(),
            }
        }
    }
    impl<C, Block, AccountId, T> AssetApi<<Block as BlockT>::Hash, AccountId, T> for Asset<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: AssetRuntimeApi<Block, AccountId, T>,
        AccountId: Codec,
        T: Codec,
    {
        fn can_transfer(
            &self,
            sender: AccountId,
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: T,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<CanTransferResult> {
            {
                let api = self.client.runtime_api();
                let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
                let result = (|api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                    api.can_transfer(at, sender, ticker, from_did, to_did, value)
                })(api, &at)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                    message: "Unable to check transfer".into(),
                    data: Some(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &[""],
                                &match (&e,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .into(),
                    ),
                })?;
                Ok(result)
            }
        }
    }
}
pub mod pips {
    pub use node_rpc_runtime_api::pips::{
        self as runtime_api, CappedVoteCount, PipsApi as PipsRuntimeApi,
    };
    use pallet_pips::{HistoricalVoting, HistoricalVotingByAddress, HistoricalVotingItem};
    use polymesh_primitives::IdentityId;
    use codec::Codec;
    use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
    use jsonrpc_derive::rpc;
    use sp_api::{ApiRef, ProvideRuntimeApi};
    use sp_blockchain::HeaderBackend;
    use frame_support::Parameter;
    use sp_runtime::{
        generic::BlockId,
        traits::{Block as BlockT, UniqueSaturatedInto},
    };
    use sp_std::{prelude::*, vec::Vec};
    use std::sync::Arc;
    mod rpc_impl_PipsApi {
        use jsonrpc_core as _jsonrpc_core;
        use super::*;
        /// The generated client module.
        pub mod gen_client {
            use jsonrpc_core_client as _jsonrpc_core_client;
            use super::*;
            use _jsonrpc_core::{
                Call, Error, ErrorCode, Id, MethodCall, Params, Request, Response, Version,
            };
            use _jsonrpc_core::futures::prelude::*;
            use _jsonrpc_core::futures::sync::{mpsc, oneshot};
            use _jsonrpc_core::serde_json::{self, Value};
            use _jsonrpc_core_client::{
                RpcChannel, RpcError, RpcFuture, TypedClient, TypedSubscriptionStream,
            };
            /// The Client.
            pub struct Client<BlockHash, AccountId, Balance> {
                inner: TypedClient,
                _0: std::marker::PhantomData<BlockHash>,
                _1: std::marker::PhantomData<AccountId>,
                _2: std::marker::PhantomData<Balance>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl<
                    BlockHash: ::core::clone::Clone,
                    AccountId: ::core::clone::Clone,
                    Balance: ::core::clone::Clone,
                > ::core::clone::Clone for Client<BlockHash, AccountId, Balance>
            {
                #[inline]
                fn clone(&self) -> Client<BlockHash, AccountId, Balance> {
                    match *self {
                        Client {
                            inner: ref __self_0_0,
                            _0: ref __self_0_1,
                            _1: ref __self_0_2,
                            _2: ref __self_0_3,
                        } => Client {
                            inner: ::core::clone::Clone::clone(&(*__self_0_0)),
                            _0: ::core::clone::Clone::clone(&(*__self_0_1)),
                            _1: ::core::clone::Clone::clone(&(*__self_0_2)),
                            _2: ::core::clone::Clone::clone(&(*__self_0_3)),
                        },
                    }
                }
            }
            impl<BlockHash, AccountId, Balance> Client<BlockHash, AccountId, Balance>
            where
                BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                AccountId: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                Balance: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
            {
                /// Creates a new `Client`.
                pub fn new(sender: RpcChannel) -> Self {
                    Client {
                        inner: sender.into(),
                        _0: std::marker::PhantomData,
                        _1: std::marker::PhantomData,
                        _2: std::marker::PhantomData,
                    }
                }
                /// Summary of votes of a proposal given by `index`
                pub fn get_votes(
                    &self,
                    index: u32,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = CappedVoteCount, Error = RpcError> {
                    let args = (index, at);
                    self.inner
                        .call_method("pips_getVotes", "CappedVoteCount", args)
                }
                /// Retrieves proposal indices started by `address`
                pub fn proposed_by(
                    &self,
                    address: AccountId,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = Vec<u32>, Error = RpcError> {
                    let args = (address, at);
                    self.inner
                        .call_method("pips_proposedBy", "Vec < u32 >", args)
                }
                /// Retrieves proposal `address` indices voted on
                pub fn voted_on(
                    &self,
                    address: AccountId,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = Vec<u32>, Error = RpcError> {
                    let args = (address, at);
                    self.inner.call_method("pips_votedOn", "Vec < u32 >", args)
                }
                /// Retrieve historical voting of `who` account.
                pub fn voting_history_by_address(
                    &self,
                    address: AccountId,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = HistoricalVoting<Balance>, Error = RpcError>
                {
                    let args = (address, at);
                    self.inner.call_method(
                        "pips_votingHistoryByAddress",
                        "HistoricalVoting < Balance >",
                        args,
                    )
                }
                /// Retrieve historical voting of `id` identity.
                pub fn voting_history_by_id(
                    &self,
                    id: IdentityId,
                    at: Option<BlockHash>,
                ) -> impl Future<Item = HistoricalVotingByAddress<Balance>, Error = RpcError>
                {
                    let args = (id, at);
                    self.inner.call_method(
                        "pips_votingHistoryById",
                        "HistoricalVotingByAddress < Balance >",
                        args,
                    )
                }
            }
            impl<BlockHash, AccountId, Balance> From<RpcChannel> for Client<BlockHash, AccountId, Balance>
            where
                BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                AccountId: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                Balance: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
            {
                fn from(channel: RpcChannel) -> Self {
                    Client::new(channel.into())
                }
            }
        }
        /// The generated server module.
        pub mod gen_server {
            use self::_jsonrpc_core::futures as _futures;
            use super::*;
            /// Pips RPC methods.
            pub trait PipsApi<BlockHash, AccountId, Balance>:
                Sized + Send + Sync + 'static
            {
                /// Summary of votes of a proposal given by `index`
                fn get_votes(&self, index: u32, at: Option<BlockHash>) -> Result<CappedVoteCount>;
                /// Retrieves proposal indices started by `address`
                fn proposed_by(
                    &self,
                    address: AccountId,
                    at: Option<BlockHash>,
                ) -> Result<Vec<u32>>;
                /// Retrieves proposal `address` indices voted on
                fn voted_on(&self, address: AccountId, at: Option<BlockHash>) -> Result<Vec<u32>>;
                /// Retrieve historical voting of `who` account.
                fn voting_history_by_address(
                    &self,
                    address: AccountId,
                    at: Option<BlockHash>,
                ) -> Result<HistoricalVoting<Balance>>;
                /// Retrieve historical voting of `id` identity.
                fn voting_history_by_id(
                    &self,
                    id: IdentityId,
                    at: Option<BlockHash>,
                ) -> Result<HistoricalVotingByAddress<Balance>>;
                /// Create an `IoDelegate`, wiring rpc calls to the trait methods.
                fn to_delegate<M: _jsonrpc_core::Metadata>(
                    self,
                ) -> _jsonrpc_core::IoDelegate<Self, M>
                where
                    BlockHash: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
                    AccountId: Send + Sync + 'static + _jsonrpc_core::serde::de::DeserializeOwned,
                    Balance: Send + Sync + 'static + _jsonrpc_core::serde::Serialize,
                {
                    let mut del = _jsonrpc_core::IoDelegate::new(self.into());
                    del.add_method("pips_getVotes", move |base, params| {
                        let method = &(Self::get_votes
                            as fn(&Self, u32, Option<BlockHash>) -> Result<CappedVoteCount>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 1usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&1usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                1usize => params
                                    .parse::<(u32,)>()
                                    .map(|(a,)| (a, None))
                                    .map_err(Into::into),
                                2usize => params
                                    .parse::<(u32, Option<BlockHash>)>()
                                    .map(|(a, b)| (a, b))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&1usize, &2usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del.add_method("pips_proposedBy", move |base, params| {
                        let method = &(Self::proposed_by
                            as fn(&Self, AccountId, Option<BlockHash>) -> Result<Vec<u32>>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 1usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&1usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                1usize => params
                                    .parse::<(AccountId,)>()
                                    .map(|(a,)| (a, None))
                                    .map_err(Into::into),
                                2usize => params
                                    .parse::<(AccountId, Option<BlockHash>)>()
                                    .map(|(a, b)| (a, b))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&1usize, &2usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del.add_method("pips_votedOn", move |base, params| {
                        let method = &(Self::voted_on
                            as fn(&Self, AccountId, Option<BlockHash>) -> Result<Vec<u32>>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 1usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&1usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                1usize => params
                                    .parse::<(AccountId,)>()
                                    .map(|(a,)| (a, None))
                                    .map_err(Into::into),
                                2usize => params
                                    .parse::<(AccountId, Option<BlockHash>)>()
                                    .map(|(a, b)| (a, b))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&1usize, &2usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del.add_method("pips_votingHistoryByAddress", move |base, params| {
                        let method = &(Self::voting_history_by_address
                            as fn(
                                &Self,
                                AccountId,
                                Option<BlockHash>,
                            )
                                -> Result<HistoricalVoting<Balance>>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 1usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&1usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                1usize => params
                                    .parse::<(AccountId,)>()
                                    .map(|(a,)| (a, None))
                                    .map_err(Into::into),
                                2usize => params
                                    .parse::<(AccountId, Option<BlockHash>)>()
                                    .map(|(a, b)| (a, b))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&1usize, &2usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del.add_method("pips_votingHistoryById", move |base, params| {
                        let method = &(Self::voting_history_by_id
                            as fn(
                                &Self,
                                IdentityId,
                                Option<BlockHash>,
                            )
                                -> Result<HistoricalVotingByAddress<Balance>>);
                        let passed_args_num = match params {
                            _jsonrpc_core::Params::Array(ref v) => Ok(v.len()),
                            _jsonrpc_core::Params::None => Ok(0),
                            _ => Err(_jsonrpc_core::Error::invalid_params(
                                "`params` should be an array",
                            )),
                        };
                        let params =
                            passed_args_num.and_then(|passed_args_num| match passed_args_num {
                                _ if passed_args_num < 1usize => {
                                    Err(_jsonrpc_core::Error::invalid_params({
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["`params` should have at least ", " argument(s)"],
                                                &match (&1usize,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    }))
                                }
                                1usize => params
                                    .parse::<(IdentityId,)>()
                                    .map(|(a,)| (a, None))
                                    .map_err(Into::into),
                                2usize => params
                                    .parse::<(IdentityId, Option<BlockHash>)>()
                                    .map(|(a, b)| (a, b))
                                    .map_err(Into::into),
                                _ => Err(_jsonrpc_core::Error::invalid_params_with_details(
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Expected from ", " to ", " parameters."],
                                                &match (&1usize, &2usize) {
                                                    (arg0, arg1) => [
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg0,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                        ::core::fmt::ArgumentV1::new(
                                                            arg1,
                                                            ::core::fmt::Display::fmt,
                                                        ),
                                                    ],
                                                },
                                            ));
                                        res
                                    },
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                                &["Got: "],
                                                &match (&passed_args_num,) {
                                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Display::fmt,
                                                    )],
                                                },
                                            ));
                                        res
                                    },
                                )),
                            });
                        match params {
                            Ok((a, b)) => {
                                use self::_futures::{Future, IntoFuture};
                                let fut = (method)(base, a, b)
                                    .into_future()
                                    .map(|value| {
                                        _jsonrpc_core::to_value(value)
                                            .expect("Expected always-serializable type; qed")
                                    })
                                    .map_err(Into::into as fn(_) -> _jsonrpc_core::Error);
                                _futures::future::Either::A(fut)
                            }
                            Err(e) => _futures::future::Either::B(_futures::failed(e)),
                        }
                    });
                    del
                }
            }
        }
    }
    pub use self::rpc_impl_PipsApi::gen_client;
    pub use self::rpc_impl_PipsApi::gen_server::PipsApi;
    /// An implementation of pips specific RPC methods.
    pub struct Pips<T, U> {
        client: Arc<T>,
        _marker: std::marker::PhantomData<U>,
    }
    impl<T, U> Pips<T, U> {
        /// Create new `Pips` with the given reference to the client.
        pub fn new(client: Arc<T>) -> Self {
            Pips {
                client,
                _marker: Default::default(),
            }
        }
    }
    impl<C, Block, AccountId, Balance> PipsApi<<Block as BlockT>::Hash, AccountId, Balance>
        for Pips<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: PipsRuntimeApi<Block, AccountId, Balance>,
        AccountId: Codec,
        Balance: Codec + UniqueSaturatedInto<u64>,
    {
        fn get_votes(
            &self,
            index: u32,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<CappedVoteCount> {
            Ok(CappedVoteCount::ProposalNotFound)
        }
        fn proposed_by(
            &self,
            address: AccountId,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<Vec<u32>> {
            {
                let api = self.client.runtime_api();
                let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
                let result = (|api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                    api.proposed_by(at, address)
                })(api, &at)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                    message: "Unable to query `proposed_by`.".into(),
                    data: Some(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &[""],
                                &match (&e,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .into(),
                    ),
                })?;
                Ok(result)
            }
        }
        fn voted_on(
            &self,
            address: AccountId,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<Vec<u32>> {
            {
                let api = self.client.runtime_api();
                let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
                let result = (|api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                    api.voted_on(at, address)
                })(api, &at)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                    message: "Unable to query `voted_on`.".into(),
                    data: Some(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &[""],
                                &match (&e,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .into(),
                    ),
                })?;
                Ok(result)
            }
        }
        fn voting_history_by_address(
            &self,
            address: AccountId,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<Vec<HistoricalVotingItem<Balance>>> {
            {
                let api = self.client.runtime_api();
                let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
                let result = (|api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                    api.voting_history_by_address(at, address)
                })(api, &at)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                    message: "Unable to query `voting_history_by_address`.".into(),
                    data: Some(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &[""],
                                &match (&e,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .into(),
                    ),
                })?;
                Ok(result)
            }
        }
        fn voting_history_by_id(
            &self,
            id: IdentityId,
            at: Option<<Block as BlockT>::Hash>,
        ) -> Result<HistoricalVotingByAddress<Balance>> {
            {
                let api = self.client.runtime_api();
                let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
                let result = (|api: ApiRef<<C as ProvideRuntimeApi<Block>>::Api>, at| {
                    api.voting_history_by_id(at, id)
                })(api, &at)
                .map_err(|e| RpcError {
                    code: ErrorCode::ServerError(crate::Error::RuntimeError as i64),
                    message: "Unable to query `voting_history_by_id`.".into(),
                    data: Some(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &[""],
                                &match (&e,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .into(),
                    ),
                })?;
                Ok(result)
            }
        }
    }
}
