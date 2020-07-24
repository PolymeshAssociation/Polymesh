#![feature(prelude_import)]
//! Runtime API definition for Runtime RPC module.
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
pub mod asset {
    //! Runtime API definition for Identity module.
    use codec::Codec;
    use polymesh_primitives::{IdentityId, Ticker};
    use sp_std::vec::Vec;
    pub type Error = Vec<u8>;
    pub type CanTransferResult = Result<u8, Error>;
    #[doc(hidden)]
    mod sp_api_hidden_includes_DECL_RUNTIME_APIS {
        pub extern crate sp_api as sp_api;
    }
    #[doc(hidden)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    pub mod runtime_decl_for_AssetApi {
        use super::*;
        /// The API to interact with Asset.
        pub trait AssetApi<
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            AccountId,
            Balance,
        >
        where
            AccountId: Codec,
            Balance: Codec,
        {
            /// Checks whether a transaction with given parameters can take place or not.
            ///
            /// # Example
            ///
            /// In this example we are checking if Alice can transfer 500 of ticket 0x01
            /// from herself (Id=0x2a) to Bob (Id=0x3905)
            ///
            /// ```ignore
            ///  curl
            ///    -H "Content-Type: application/json"
            ///    -d {
            ///        "id":1, "jsonrpc":"2.0",
            ///        "method": "asset_canTransfer",
            ///        "params":[
            ///            "5CoRaw9Ex4DUjGcnPbPBnc2nez5ZeTmM5WL3ZDVLZzM6eEgE",
            ///            "0x010000000000000000000000",
            ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
            ///            "0x3905000000000000000000000000000000000000000000000000000000000000",
            ///            500]}
            ///    http://localhost:9933 | python3 -m json.tool
            /// ```
            fn can_transfer(
                sender: AccountId,
                ticker: Ticker,
                from_did: Option<IdentityId>,
                to_did: Option<IdentityId>,
                value: Balance,
            ) -> CanTransferResult;
        }
        pub const VERSION: u32 = 1u32;
        pub const ID: [u8; 8] = [187u8, 107u8, 169u8, 5u8, 60u8, 92u8, 157u8, 120u8];
        #[cfg(any(feature = "std", test))]
        fn convert_between_block_types<
            I: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode,
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode,
        >(
            input: &I,
            error_desc: &'static str,
        ) -> std::result::Result<R, String> {
            <R as self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode>::decode(
                &mut &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(input)
                    [..],
            )
            .map_err(|e| {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " "],
                    &match (&error_desc, &e.what()) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            })
        }
        #[cfg(any(feature = "std", test))]
        pub fn can_transfer_native_call_generator<
            'a,
            ApiImpl: AssetApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            sender: AccountId,
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: Balance,
        ) -> impl FnOnce() -> std::result::Result<CanTransferResult, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec,
        {
            move || {
                let res = ApiImpl::can_transfer(sender, ticker, from_did, to_did, value);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn can_transfer_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "AssetApi_can_transfer",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
    }
    /// The API to interact with Asset.
    #[cfg(any(feature = "std", test))]
    pub trait AssetApi<
        Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
        AccountId,
        Balance,
    >: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block>
    where
        AccountId: Codec,
        Balance: Codec,
    {
        /// Checks whether a transaction with given parameters can take place or not.
        ///
        /// # Example
        ///
        /// In this example we are checking if Alice can transfer 500 of ticket 0x01
        /// from herself (Id=0x2a) to Bob (Id=0x3905)
        ///
        /// ```ignore
        ///  curl
        ///    -H "Content-Type: application/json"
        ///    -d {
        ///        "id":1, "jsonrpc":"2.0",
        ///        "method": "asset_canTransfer",
        ///        "params":[
        ///            "5CoRaw9Ex4DUjGcnPbPBnc2nez5ZeTmM5WL3ZDVLZzM6eEgE",
        ///            "0x010000000000000000000000",
        ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
        ///            "0x3905000000000000000000000000000000000000000000000000000000000000",
        ///            500]}
        ///    http://localhost:9933 | python3 -m json.tool
        /// ```
        fn can_transfer(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            sender: AccountId,
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: Balance,
        ) -> std::result::Result<CanTransferResult, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(
                    &sender, &ticker, &from_did, &to_did, &value,
                ));
            self . AssetApi_can_transfer_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( sender , ticker , from_did , to_did , value ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < CanTransferResult as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "can_transfer" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Checks whether a transaction with given parameters can take place or not.
        ///
        /// # Example
        ///
        /// In this example we are checking if Alice can transfer 500 of ticket 0x01
        /// from herself (Id=0x2a) to Bob (Id=0x3905)
        ///
        /// ```ignore
        ///  curl
        ///    -H "Content-Type: application/json"
        ///    -d {
        ///        "id":1, "jsonrpc":"2.0",
        ///        "method": "asset_canTransfer",
        ///        "params":[
        ///            "5CoRaw9Ex4DUjGcnPbPBnc2nez5ZeTmM5WL3ZDVLZzM6eEgE",
        ///            "0x010000000000000000000000",
        ///            "0x2a00000000000000000000000000000000000000000000000000000000000000",
        ///            "0x3905000000000000000000000000000000000000000000000000000000000000",
        ///            500]}
        ///    http://localhost:9933 | python3 -m json.tool
        /// ```
        fn can_transfer_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            sender: AccountId,
            ticker: Ticker,
            from_did: Option<IdentityId>,
            to_did: Option<IdentityId>,
            value: Balance,
        ) -> std::result::Result<CanTransferResult, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(
                    &sender, &ticker, &from_did, &to_did, &value,
                ));
            self . AssetApi_can_transfer_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( sender , ticker , from_did , to_did , value ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < CanTransferResult as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "can_transfer" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn AssetApi_can_transfer_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(
                AccountId,
                Ticker,
                Option<IdentityId>,
                Option<IdentityId>,
                Balance,
            )>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<
                CanTransferResult,
            >,
            Self::Error,
        >;
    }
    #[cfg(any(feature = "std", test))]
    impl<
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            AccountId,
            Balance,
            __Sr_Api_Error__,
        > self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::RuntimeApiInfo
        for AssetApi<Block, AccountId, Balance, Error = __Sr_Api_Error__>
    {
        const ID: [u8; 8] = [187u8, 107u8, 169u8, 5u8, 60u8, 92u8, 157u8, 120u8];
        const VERSION: u32 = 1u32;
    }
}
pub mod pips {
    //! Runtime API definition for pips module.
    use pallet_pips::{HistoricalVoting, HistoricalVotingByAddress, VoteCount};
    use polymesh_primitives::IdentityId;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};
    use codec::{Codec, Decode, Encode, EncodeLike};
    use frame_support::Parameter;
    use sp_runtime::traits::{SaturatedConversion, UniqueSaturatedInto};
    use sp_std::{prelude::*, vec::Vec};
    /// A capped version of `VoteCount`.
    ///
    /// The `Balance` is capped (or expanded) to `u64` to avoid serde issues with `u128`.
    #[serde(rename_all = "camelCase")]
    pub enum CappedVoteCount {
        /// Proposal was found and has the following votes.
        Success {
            /// Stake for
            ayes: u64,
            /// Stake against
            nays: u64,
        },
        /// Proposal was not for given index.
        ProposalNotFound,
    }
    impl ::core::marker::StructuralEq for CappedVoteCount {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for CappedVoteCount {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<u64>;
                let _: ::core::cmp::AssertParamIsEq<u64>;
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for CappedVoteCount {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for CappedVoteCount {
        #[inline]
        fn eq(&self, other: &CappedVoteCount) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                let __arg_1_vi =
                    unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &CappedVoteCount::Success {
                                ayes: ref __self_0,
                                nays: ref __self_1,
                            },
                            &CappedVoteCount::Success {
                                ayes: ref __arg_1_0,
                                nays: ref __arg_1_1,
                            },
                        ) => (*__self_0) == (*__arg_1_0) && (*__self_1) == (*__arg_1_1),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &CappedVoteCount) -> bool {
            {
                let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                let __arg_1_vi =
                    unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &CappedVoteCount::Success {
                                ayes: ref __self_0,
                                nays: ref __self_1,
                            },
                            &CappedVoteCount::Success {
                                ayes: ref __arg_1_0,
                                nays: ref __arg_1_1,
                            },
                        ) => (*__self_0) != (*__arg_1_0) || (*__self_1) != (*__arg_1_1),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl _parity_scale_codec::Encode for CappedVoteCount {
            fn encode_to<EncOut: _parity_scale_codec::Output>(&self, dest: &mut EncOut) {
                match *self {
                    CappedVoteCount::Success { ref ayes, ref nays } => {
                        dest.push_byte(0usize as u8);
                        dest.push(ayes);
                        dest.push(nays);
                    }
                    CappedVoteCount::ProposalNotFound => {
                        dest.push_byte(1usize as u8);
                    }
                    _ => (),
                }
            }
        }
        impl _parity_scale_codec::EncodeLike for CappedVoteCount {}
    };
    const _: () =
        {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate codec as _parity_scale_codec;
            impl _parity_scale_codec::Decode for CappedVoteCount {
                fn decode<DecIn: _parity_scale_codec::Input>(
                    input: &mut DecIn,
                ) -> core::result::Result<Self, _parity_scale_codec::Error> {
                    match input.read_byte()? {
                        x if x == 0usize as u8 => {
                            Ok(CappedVoteCount::Success {
                                ayes: {
                                    let res = _parity_scale_codec::Decode::decode(input);
                                    match res {
                                        Err(_) => return Err(
                                            "Error decoding field CappedVoteCount :: Success.ayes"
                                                .into(),
                                        ),
                                        Ok(a) => a,
                                    }
                                },
                                nays: {
                                    let res = _parity_scale_codec::Decode::decode(input);
                                    match res {
                                        Err(_) => return Err(
                                            "Error decoding field CappedVoteCount :: Success.nays"
                                                .into(),
                                        ),
                                        Ok(a) => a,
                                    }
                                },
                            })
                        }
                        x if x == 1usize as u8 => Ok(CappedVoteCount::ProposalNotFound),
                        x => Err("No such variant in enum CappedVoteCount".into()),
                    }
                }
            }
        };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for CappedVoteCount {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&CappedVoteCount::Success {
                    ayes: ref __self_0,
                    nays: ref __self_1,
                },) => {
                    let mut debug_trait_builder = f.debug_struct("Success");
                    let _ = debug_trait_builder.field("ayes", &&(*__self_0));
                    let _ = debug_trait_builder.field("nays", &&(*__self_1));
                    debug_trait_builder.finish()
                }
                (&CappedVoteCount::ProposalNotFound,) => {
                    let mut debug_trait_builder = f.debug_tuple("ProposalNotFound");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_SERIALIZE_FOR_CappedVoteCount: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CappedVoteCount {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    CappedVoteCount::Success { ref ayes, ref nays } => {
                        let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                            __serializer,
                            "CappedVoteCount",
                            0u32,
                            "success",
                            0 + 1 + 1,
                        ) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "ayes",
                            ayes,
                        ) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "nays",
                            nays,
                        ) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStructVariant::end(__serde_state)
                    }
                    CappedVoteCount::ProposalNotFound => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "CappedVoteCount",
                            1u32,
                            "proposalNotFound",
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _IMPL_DESERIALIZE_FOR_CappedVoteCount: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CappedVoteCount {
            fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                enum __Field {
                    __field0,
                    __field1,
                }
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::export::Formatter,
                    ) -> _serde::export::fmt::Result {
                        _serde::export::Formatter::write_str(__formatter, "variant identifier")
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::export::Ok(__Field::__field0),
                            1u64 => _serde::export::Ok(__Field::__field1),
                            _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                _serde::de::Unexpected::Unsigned(__value),
                                &"variant index 0 <= i < 2",
                            )),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "success" => _serde::export::Ok(__Field::__field0),
                            "proposalNotFound" => _serde::export::Ok(__Field::__field1),
                            _ => _serde::export::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            )),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::export::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"success" => _serde::export::Ok(__Field::__field0),
                            b"proposalNotFound" => _serde::export::Ok(__Field::__field1),
                            _ => {
                                let __value = &_serde::export::from_utf8_lossy(__value);
                                _serde::export::Err(_serde::de::Error::unknown_variant(
                                    __value, VARIANTS,
                                ))
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::export::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                    }
                }
                struct __Visitor<'de> {
                    marker: _serde::export::PhantomData<CappedVoteCount>,
                    lifetime: _serde::export::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CappedVoteCount;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::export::Formatter,
                    ) -> _serde::export::fmt::Result {
                        _serde::export::Formatter::write_str(__formatter, "enum CappedVoteCount")
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::export::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match match _serde::de::EnumAccess::variant(__data) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            (__Field::__field0, __variant) => {
                                #[allow(non_camel_case_types)]
                                enum __Field {
                                    __field0,
                                    __field1,
                                    __ignore,
                                }
                                struct __FieldVisitor;
                                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                    type Value = __Field;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::export::Formatter,
                                    ) -> _serde::export::fmt::Result
                                    {
                                        _serde::export::Formatter::write_str(
                                            __formatter,
                                            "field identifier",
                                        )
                                    }
                                    fn visit_u64<__E>(
                                        self,
                                        __value: u64,
                                    ) -> _serde::export::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            0u64 => _serde::export::Ok(__Field::__field0),
                                            1u64 => _serde::export::Ok(__Field::__field1),
                                            _ => _serde::export::Err(
                                                _serde::de::Error::invalid_value(
                                                    _serde::de::Unexpected::Unsigned(__value),
                                                    &"field index 0 <= i < 2",
                                                ),
                                            ),
                                        }
                                    }
                                    fn visit_str<__E>(
                                        self,
                                        __value: &str,
                                    ) -> _serde::export::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            "ayes" => _serde::export::Ok(__Field::__field0),
                                            "nays" => _serde::export::Ok(__Field::__field1),
                                            _ => _serde::export::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_bytes<__E>(
                                        self,
                                        __value: &[u8],
                                    ) -> _serde::export::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            b"ayes" => _serde::export::Ok(__Field::__field0),
                                            b"nays" => _serde::export::Ok(__Field::__field1),
                                            _ => _serde::export::Ok(__Field::__ignore),
                                        }
                                    }
                                }
                                impl<'de> _serde::Deserialize<'de> for __Field {
                                    #[inline]
                                    fn deserialize<__D>(
                                        __deserializer: __D,
                                    ) -> _serde::export::Result<Self, __D::Error>
                                    where
                                        __D: _serde::Deserializer<'de>,
                                    {
                                        _serde::Deserializer::deserialize_identifier(
                                            __deserializer,
                                            __FieldVisitor,
                                        )
                                    }
                                }
                                struct __Visitor<'de> {
                                    marker: _serde::export::PhantomData<CappedVoteCount>,
                                    lifetime: _serde::export::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = CappedVoteCount;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::export::Formatter,
                                    ) -> _serde::export::fmt::Result
                                    {
                                        _serde::export::Formatter::write_str(
                                            __formatter,
                                            "struct variant CappedVoteCount::Success",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::export::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 =
                                            match match _serde::de::SeqAccess::next_element::<u64>(
                                                &mut __seq,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            } {
                                                _serde::export::Some(__value) => __value,
                                                _serde::export::None => {
                                                    return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct variant CappedVoteCount::Success with 2 elements" ) ) ;
                                                }
                                            };
                                        let __field1 =
                                            match match _serde::de::SeqAccess::next_element::<u64>(
                                                &mut __seq,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            } {
                                                _serde::export::Some(__value) => __value,
                                                _serde::export::None => {
                                                    return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 1usize , & "struct variant CappedVoteCount::Success with 2 elements" ) ) ;
                                                }
                                            };
                                        _serde::export::Ok(CappedVoteCount::Success {
                                            ayes: __field0,
                                            nays: __field1,
                                        })
                                    }
                                    #[inline]
                                    fn visit_map<__A>(
                                        self,
                                        mut __map: __A,
                                    ) -> _serde::export::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::MapAccess<'de>,
                                    {
                                        let mut __field0: _serde::export::Option<u64> =
                                            _serde::export::None;
                                        let mut __field1: _serde::export::Option<u64> =
                                            _serde::export::None;
                                        while let _serde::export::Some(__key) =
                                            match _serde::de::MapAccess::next_key::<__Field>(
                                                &mut __map,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            }
                                        {
                                            match __key {
                                                __Field::__field0 => {
                                                    if _serde::export::Option::is_some(&__field0) {
                                                        return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "ayes" ) ) ;
                                                    }
                                                    __field0 = _serde::export::Some(
                                                        match _serde::de::MapAccess::next_value::<u64>(
                                                            &mut __map,
                                                        ) {
                                                            _serde::export::Ok(__val) => __val,
                                                            _serde::export::Err(__err) => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    );
                                                }
                                                __Field::__field1 => {
                                                    if _serde::export::Option::is_some(&__field1) {
                                                        return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "nays" ) ) ;
                                                    }
                                                    __field1 = _serde::export::Some(
                                                        match _serde::de::MapAccess::next_value::<u64>(
                                                            &mut __map,
                                                        ) {
                                                            _serde::export::Ok(__val) => __val,
                                                            _serde::export::Err(__err) => {
                                                                return _serde::export::Err(__err);
                                                            }
                                                        },
                                                    );
                                                }
                                                _ => {
                                                    let _ = match _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(
                                                        &mut __map
                                                    ) {
                                                        _serde::export::Ok(__val) => __val,
                                                        _serde::export::Err(__err) => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    };
                                                }
                                            }
                                        }
                                        let __field0 = match __field0 {
                                            _serde::export::Some(__field0) => __field0,
                                            _serde::export::None => {
                                                match _serde::private::de::missing_field("ayes") {
                                                    _serde::export::Ok(__val) => __val,
                                                    _serde::export::Err(__err) => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                }
                                            }
                                        };
                                        let __field1 = match __field1 {
                                            _serde::export::Some(__field1) => __field1,
                                            _serde::export::None => {
                                                match _serde::private::de::missing_field("nays") {
                                                    _serde::export::Ok(__val) => __val,
                                                    _serde::export::Err(__err) => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                }
                                            }
                                        };
                                        _serde::export::Ok(CappedVoteCount::Success {
                                            ayes: __field0,
                                            nays: __field1,
                                        })
                                    }
                                }
                                const FIELDS: &'static [&'static str] = &["ayes", "nays"];
                                _serde::de::VariantAccess::struct_variant(
                                    __variant,
                                    FIELDS,
                                    __Visitor {
                                        marker: _serde::export::PhantomData::<CappedVoteCount>,
                                        lifetime: _serde::export::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field1, __variant) => {
                                match _serde::de::VariantAccess::unit_variant(__variant) {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                                _serde::export::Ok(CappedVoteCount::ProposalNotFound)
                            }
                        }
                    }
                }
                const VARIANTS: &'static [&'static str] = &["success", "proposalNotFound"];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "CappedVoteCount",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::export::PhantomData::<CappedVoteCount>,
                        lifetime: _serde::export::PhantomData,
                    },
                )
            }
        }
    };
    impl CappedVoteCount {
        /// Create a new `CappedVoteCount` from `VoteCount`.
        pub fn new<Balance: UniqueSaturatedInto<u64>>(count: VoteCount<Balance>) -> Self {
            match count {
                VoteCount::Success { ayes, nays } => CappedVoteCount::Success {
                    ayes: ayes.saturated_into(),
                    nays: nays.saturated_into(),
                },
                VoteCount::ProposalNotFound => CappedVoteCount::ProposalNotFound,
            }
        }
    }
    #[doc(hidden)]
    mod sp_api_hidden_includes_DECL_RUNTIME_APIS {
        pub extern crate sp_api as sp_api;
    }
    #[doc(hidden)]
    #[allow(dead_code)]
    #[allow(deprecated)]
    pub mod runtime_decl_for_PipsApi {
        use super::*;
        /// The API to interact with Pips governance.
        pub trait PipsApi<
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            AccountId,
            Balance,
        >
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            /// Retrieve votes for a proposal for a given `pips_index`.
            fn get_votes(pips_index: u32) -> VoteCount<Balance>;
            /// Retrieve proposals started by `address`.
            fn proposed_by(address: AccountId) -> Vec<u32>;
            /// Retrieve proposals `address` voted on.
            fn voted_on(address: AccountId) -> Vec<u32>;
            /// Retrieve referendums voted on information by `address` account.
            fn voting_history_by_address(address: AccountId) -> HistoricalVoting<Balance>;
            /// Retrieve referendums voted on information by `id` identity (and its signing items).
            fn voting_history_by_id(id: IdentityId) -> HistoricalVotingByAddress<Balance>;
        }
        pub const VERSION: u32 = 1u32;
        pub const ID: [u8; 8] = [50u8, 147u8, 66u8, 153u8, 71u8, 115u8, 4u8, 127u8];
        #[cfg(any(feature = "std", test))]
        fn convert_between_block_types<
            I: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode,
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode,
        >(
            input: &I,
            error_desc: &'static str,
        ) -> std::result::Result<R, String> {
            <R as self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode>::decode(
                &mut &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(input)
                    [..],
            )
            .map_err(|e| {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " "],
                    &match (&error_desc, &e.what()) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            })
        }
        #[cfg(any(feature = "std", test))]
        pub fn get_votes_native_call_generator<
            'a,
            ApiImpl: PipsApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            pips_index: u32,
        ) -> impl FnOnce() -> std::result::Result<VoteCount<Balance>, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            move || {
                let res = ApiImpl::get_votes(pips_index);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn proposed_by_native_call_generator<
            'a,
            ApiImpl: PipsApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            address: AccountId,
        ) -> impl FnOnce() -> std::result::Result<Vec<u32>, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            move || {
                let res = ApiImpl::proposed_by(address);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn voted_on_native_call_generator<
            'a,
            ApiImpl: PipsApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            address: AccountId,
        ) -> impl FnOnce() -> std::result::Result<Vec<u32>, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            move || {
                let res = ApiImpl::voted_on(address);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn voting_history_by_address_native_call_generator<
            'a,
            ApiImpl: PipsApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            address: AccountId,
        ) -> impl FnOnce() -> std::result::Result<HistoricalVoting<Balance>, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            move || {
                let res = ApiImpl::voting_history_by_address(address);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn voting_history_by_id_native_call_generator<
            'a,
            ApiImpl: PipsApi<Block, AccountId, Balance>,
            NodeBlock: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT + 'a,
            AccountId: 'a,
            Balance: 'a,
        >(
            id: IdentityId,
        ) -> impl FnOnce() -> std::result::Result<HistoricalVotingByAddress<Balance>, String> + 'a
        where
            AccountId: Codec,
            Balance: Codec + UniqueSaturatedInto<u128>,
        {
            move || {
                let res = ApiImpl::voting_history_by_id(id);
                Ok(res)
            }
        }
        #[cfg(any(feature = "std", test))]
        pub fn get_votes_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "PipsApi_get_votes",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
        #[cfg(any(feature = "std", test))]
        pub fn proposed_by_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "PipsApi_proposed_by",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
        #[cfg(any(feature = "std", test))]
        pub fn voted_on_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "PipsApi_voted_on",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
        #[cfg(any(feature = "std", test))]
        pub fn voting_history_by_address_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "PipsApi_voting_history_by_address",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
        #[cfg(any(feature = "std", test))]
        pub fn voting_history_by_id_call_api_at<
            R: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode
                + self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Decode
                + PartialEq,
            NC: FnOnce() -> std::result::Result<R, String> + std::panic::UnwindSafe,
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            T: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAt<Block>,
            C: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block, Error = T::Error>,
        >(
            call_runtime_at: &T,
            core_api: &C,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            args: Vec<u8>,
            changes: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::OverlayedChanges,
            >,
            storage_transaction_cache: &std::cell::RefCell<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::StorageTransactionCache<
                    Block,
                    T::StateBackend,
                >,
            >,
            initialized_block: &std::cell::RefCell<
                Option<self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>>,
            >,
            native_call: Option<NC>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            recorder: &Option<
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ProofRecorder<Block>,
            >,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<R>,
            T::Error,
        > {
            let version = call_runtime_at.runtime_version_at(at)?;
            use self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::InitializeBlock;
            let initialize_block = if false {
                InitializeBlock::Skip
            } else {
                InitializeBlock::Do(&initialized_block)
            };
            let update_initialized_block = || ();
            let params = self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::CallApiAtParams {
                core_api,
                at,
                function: "PipsApi_voting_history_by_id",
                native_call,
                arguments: args,
                overlayed_changes: changes,
                storage_transaction_cache,
                initialize_block,
                context,
                recorder,
            };
            let ret = call_runtime_at.call_api_at(params)?;
            update_initialized_block();
            Ok(ret)
        }
    }
    /// The API to interact with Pips governance.
    #[cfg(any(feature = "std", test))]
    pub trait PipsApi<
        Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
        AccountId,
        Balance,
    >: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Core<Block>
    where
        AccountId: Codec,
        Balance: Codec + UniqueSaturatedInto<u128>,
    {
        /// Retrieve votes for a proposal for a given `pips_index`.
        fn get_votes(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            pips_index: u32,
        ) -> std::result::Result<VoteCount<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(
                    &(&pips_index),
                );
            self . PipsApi_get_votes_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( pips_index ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < VoteCount < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "get_votes" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Retrieve votes for a proposal for a given `pips_index`.
        fn get_votes_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            pips_index: u32,
        ) -> std::result::Result<VoteCount<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(
                    &(&pips_index),
                );
            self . PipsApi_get_votes_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( pips_index ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < VoteCount < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "get_votes" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn PipsApi_get_votes_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(u32)>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<
                VoteCount<Balance>,
            >,
            Self::Error,
        >;
        /// Retrieve proposals started by `address`.
        fn proposed_by(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            address: AccountId,
        ) -> std::result::Result<Vec<u32>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_proposed_by_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < Vec < u32 > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "proposed_by" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Retrieve proposals started by `address`.
        fn proposed_by_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            address: AccountId,
        ) -> std::result::Result<Vec<u32>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_proposed_by_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < Vec < u32 > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "proposed_by" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn PipsApi_proposed_by_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(AccountId)>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<Vec<u32>>,
            Self::Error,
        >;
        /// Retrieve proposals `address` voted on.
        fn voted_on(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            address: AccountId,
        ) -> std::result::Result<Vec<u32>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_voted_on_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < Vec < u32 > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voted_on" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Retrieve proposals `address` voted on.
        fn voted_on_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            address: AccountId,
        ) -> std::result::Result<Vec<u32>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_voted_on_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < Vec < u32 > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voted_on" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn PipsApi_voted_on_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(AccountId)>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<Vec<u32>>,
            Self::Error,
        >;
        /// Retrieve referendums voted on information by `address` account.
        fn voting_history_by_address(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            address: AccountId,
        ) -> std::result::Result<HistoricalVoting<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_voting_history_by_address_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < HistoricalVoting < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voting_history_by_address" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Retrieve referendums voted on information by `address` account.
        fn voting_history_by_address_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            address: AccountId,
        ) -> std::result::Result<HistoricalVoting<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&address));
            self . PipsApi_voting_history_by_address_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( address ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < HistoricalVoting < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voting_history_by_address" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn PipsApi_voting_history_by_address_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(AccountId)>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<
                HistoricalVoting<Balance>,
            >,
            Self::Error,
        >;
        /// Retrieve referendums voted on information by `id` identity (and its signing items).
        fn voting_history_by_id(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            id: IdentityId,
        ) -> std::result::Result<HistoricalVotingByAddress<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&id));
            self . PipsApi_voting_history_by_id_runtime_api_impl ( __runtime_api_at_param__ , self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: ExecutionContext :: OffchainCall ( None ) , Some ( ( id ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < HistoricalVotingByAddress < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voting_history_by_id" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        /// Retrieve referendums voted on information by `id` identity (and its signing items).
        fn voting_history_by_id_with_context(
            &self,
            __runtime_api_at_param__ : & self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: BlockId < Block >,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            id: IdentityId,
        ) -> std::result::Result<HistoricalVotingByAddress<Balance>, Self::Error> {
            let runtime_api_impl_params_encoded =
                self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::Encode::encode(&(&id));
            self . PipsApi_voting_history_by_id_runtime_api_impl ( __runtime_api_at_param__ , context , Some ( ( id ) ) , runtime_api_impl_params_encoded ) . and_then ( | r | match r { self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Native ( n ) => { Ok ( n ) } self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: NativeOrEncoded :: Encoded ( r ) => { < HistoricalVotingByAddress < Balance > as self :: sp_api_hidden_includes_DECL_RUNTIME_APIS :: sp_api :: Decode > :: decode ( & mut & r [ .. ] ) . map_err ( | err | { let res = :: alloc :: fmt :: format ( :: core :: fmt :: Arguments :: new_v1 ( & [ "Failed to decode result of `" , "`: " ] , & match ( & "voting_history_by_id" , & err . what ( ) ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Display :: fmt ) ] , } ) ) ; res } . into ( ) ) } } )
        }
        #[doc(hidden)]
        fn PipsApi_voting_history_by_id_runtime_api_impl(
            &self,
            at: &self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockId<Block>,
            context: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::ExecutionContext,
            params: Option<(IdentityId)>,
            params_encoded: Vec<u8>,
        ) -> std::result::Result<
            self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::NativeOrEncoded<
                HistoricalVotingByAddress<Balance>,
            >,
            Self::Error,
        >;
    }
    #[cfg(any(feature = "std", test))]
    impl<
            Block: self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::BlockT,
            AccountId,
            Balance,
            __Sr_Api_Error__,
        > self::sp_api_hidden_includes_DECL_RUNTIME_APIS::sp_api::RuntimeApiInfo
        for PipsApi<Block, AccountId, Balance, Error = __Sr_Api_Error__>
    {
        const ID: [u8; 8] = [50u8, 147u8, 66u8, 153u8, 71u8, 115u8, 4u8, 127u8];
        const VERSION: u32 = 1u32;
    }
}
