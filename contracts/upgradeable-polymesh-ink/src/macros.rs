#[macro_export]
macro_rules! upgradable_api {
    (
        $(#[$mod_attr:meta])*
        mod $mod_name:ident {
            impl $api_type:ident {
                $(
                    $(#[doc = $doc_attr:tt])*
                    $(#[ink($ink_attr:tt)])*
                    $fn_vis:vis fn $fn_name:ident(
                        & $self:ident
                        $(, $param:ident: $ty:ty)*
                    ) -> $fn_return:ty {
                        $( $fn_impl:tt )*
                    }
                )*
            }
        }
    ) => {
        pub use $mod_name::*;

        #[cfg_attr(not(feature = "as-library"), ink::contract(env = PolymeshEnvironment))]
        #[cfg(not(feature = "as-library"))]
        mod $mod_name {
            use super::*;

            #[ink(storage)]
            pub struct $api_type {
            }

            impl $api_type {
                 #[ink(constructor)]
                 pub fn new() -> Self {
                     panic!("Only upload this contract, don't deploy it.");
                 }
             }

            impl $api_type {
                $(
                    $(#[doc = $doc_attr])*
                    $(#[ink($ink_attr)])*
                    $fn_vis fn $fn_name(&self $(, $param: $ty)*) -> $fn_return {
                        ::paste::paste! {
                            self.[<__impl_ $fn_name>]($($param),*)
                        }
                    }
                )*
            }

            impl $api_type {
                $(
                    ::paste::paste! {
                        fn [<__impl_ $fn_name>](&$self $(, $param: $ty)*) -> $fn_return {
                            $( $fn_impl )*
                        }
                    }
                )*
            }
        }

        #[cfg(feature = "as-library")]
        mod $mod_name {
            use super::*;

            #[cfg(not(feature = "always-delegate"))]
            pub type UpgradeHash = Option<Hash>;
            #[cfg(feature = "always-delegate")]
            pub type UpgradeHash = Hash;

            /// Upgradable wrapper for the Polymesh Runtime API.
            ///
            /// Contracts can use this to maintain support accross
            /// major Polymesh releases.
            #[derive(Debug, Default, scale::Encode, scale::Decode)]
            #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
            #[derive(ink_storage::traits::SpreadLayout)]
            #[derive(ink_storage::traits::PackedLayout)]
            #[derive(ink_storage::traits::SpreadAllocate)]
            #[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
            pub struct $api_type {
                hash: UpgradeHash,
                #[cfg(feature = "tracker")]
                tracker: Option<UpgradeTrackerRef>,
            }

            impl $api_type {
                #[cfg(not(feature = "tracker"))]
                pub fn new(hash: UpgradeHash) -> Self {
                    Self { hash }
                }

                #[cfg(feature = "tracker")]
                pub fn new(hash: UpgradeHash, tracker: Option<UpgradeTrackerRef>) -> Self {
                    Self { hash, tracker }
                }

                #[cfg(feature = "tracker")]
                pub fn new_tracker(tracker: UpgradeTrackerRef) -> Self {
                    #[cfg(not(feature = "always-delegate"))]
                    let hash = tracker.get_latest_upgrade(API_VERSION).ok();
                    #[cfg(feature = "always-delegate")]
                    let hash = tracker.get_latest_upgrade(API_VERSION).unwrap();
                    Self { hash, tracker: Some(tracker) }
                }

                /// Update code hash.
                pub fn update_code_hash(&mut self, hash: UpgradeHash) {
                    self.hash = hash;
                }

                #[cfg(feature = "tracker")]
                pub fn check_for_upgrade(&mut self) -> Result<(), UpgradeError> {
                    if let Some(tracker) = &self.tracker {
                        #[cfg(not(feature = "always-delegate"))]
                        {
                            self.hash = tracker.get_latest_upgrade(API_VERSION).ok();
                        }
                        #[cfg(feature = "always-delegate")]
                        {
                            self.hash = tracker.get_latest_upgrade(API_VERSION)?;
                        }
                    }
                    Ok(())
                }
            }

            impl $api_type {
                $(
                    $crate::upgradable_api! {
                        @impl_api_func
                        $(#[doc = $doc_attr])*
                        $(#[ink($ink_attr)])*
                        $fn_vis fn $fn_name(
                            &$self
                            $(, $param: $ty)*
                        ) -> $fn_return {
                            $( $fn_impl )*
                        }
                    }
                )*
            }
        }
    };
    // Upgradable api method.
    (@impl_api_func
        $(#[doc = $doc_attr:tt])*
        $(#[ink($ink_attr:tt)])*
        $fn_vis:vis fn $fn_name:ident(
            &$self:ident
            $(, $param:ident: $ty:ty)*
        ) -> $fn_return:ty {
            $( $fn_impl:tt )*
        }
    ) => {
        $(#[doc = $doc_attr])*
        $fn_vis fn $fn_name(&$self $(, $param: $ty)*) -> $fn_return {
            use ink_env::call::{DelegateCall, ExecutionInput, Selector};
            #[cfg(not(feature = "always-delegate"))]
            let hash = $self.hash;
            #[cfg(feature = "always-delegate")]
            let hash = Some($self.hash);
            if let Some(hash) = hash {
                const FUNC: &str = stringify!($func);
                let selector: [u8; 4] = ::polymesh_api::ink::blake2_256(FUNC.as_bytes())[..4]
                  .try_into().unwrap();
                let ret = ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                    .call_type(DelegateCall::new().code_hash(hash))
                    .exec_input(
                        ExecutionInput::new(Selector::new(selector))
                            .push_arg(($($param),*)),
                    )
                    .returns::<$fn_return>()
                    .fire()
                    .unwrap_or_else(|err| panic!("delegate call to {:?} failed due to {:?}", hash, err))?;
                Ok(ret)
            } else {
                $( $fn_impl )*
            }
        }
    };
    // Non-upgradable api method.
    (@impl_api_func
        $(#[doc = $doc_attr:tt])*
        $fn_vis:vis fn $fn_name:ident(
            &$self:ident
            $(, $param:ident: $ty:ty)*
        ) -> $fn_return:ty {
            $( $fn_impl:tt )*
        }
    ) => {
        $(#[doc = $doc_attr])*
        $fn_vis fn $fn_name(&$self $(, $param: $ty)*) -> $fn_return {
            $( $fn_impl )*
        }
    };
}
