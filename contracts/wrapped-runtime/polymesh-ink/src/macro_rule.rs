#[macro_export]
macro_rules! upgradable_api {
    (
        $(#[$mod_attr:meta])*
        mod $mod_name:ident {
            impl $api_type:ident {
                $(
                    $(#[doc = $doc_attr:tt])*
                    #[ink(message)]
                    $(#[$fn_attr:meta])*
                    pub fn $fn_name:ident(
                        $(& mut $mut_self:ident)*
                        $(& $self:ident)*
                        $(, $param:ident: $ty:ty)+
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
                    #[ink(message)]
                    $(#[$fn_attr])*
                    pub fn $fn_name(
                        $(& mut $mut_self)*
                        $(& $self)*
                        $(, $param: $ty)+
                    ) -> $fn_return {
                        $( $fn_impl )*
                    }
                )*
            }
        }

        #[cfg(feature = "as-library")]
        mod $mod_name {
            use super::*;

            /// Contracts would store this a value of this type.
            #[derive(Debug, Default, scale::Encode, scale::Decode)]
            #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
            #[derive(ink_storage::traits::SpreadLayout)]
            #[derive(ink_storage::traits::PackedLayout)]
            #[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
            pub struct $api_type {
                hash: Option<Hash>,
                #[cfg(feature = "tracker")]
                tracker: Option<UpgradeTrackerRef>,
            }

            impl $api_type {
                #[cfg(not(feature = "tracker"))]
                pub fn new(hash: Option<Hash>) -> Self {
                    Self { hash }
                }
            
                #[cfg(feature = "tracker")]
                pub fn new(hash: Option<Hash>, tracker: Option<UpgradeTrackerRef>) -> Self {
                    Self { hash, tracker }
                }
            
                /// Update code hash.
                pub fn update_code_hash(&mut self, hash: Option<Hash>) {
                    self.hash = hash;
                }
            
                #[cfg(feature = "tracker")]
                pub fn check_for_upgrade(&mut self) {
                    if let Some(tracker) = &self.tracker {
                        self.hash = tracker.get_latest_upgrade(API_VERSION);
                    }
                }
            }

            impl $api_type {
                $(
                    $(#[doc = $doc_attr])*
                    pub fn $fn_name(&self, $($param: $ty),+) -> $fn_return {
                        use ink_env::call::{DelegateCall, ExecutionInput, Selector};
                        if let Some(hash) = self.hash {
                            const FUNC: &str = stringify!($func);
                            let selector: [u8; 4] = ::polymesh_api::ink::blake2_256(FUNC.as_bytes())[..4]
                              .try_into().unwrap();
                            let ret = ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                                .call_type(DelegateCall::new().code_hash(hash))
                                .exec_input(
                                    ExecutionInput::new(Selector::new(selector))
                                        .push_arg(($($param),+)),
                                )
                                .returns::<$fn_return>()
                                .fire()
                                .unwrap_or_else(|err| panic!("delegate call to {:?} failed due to {:?}", hash, err))?;
                            Ok(ret)
                        } else {
                            $( $fn_impl )*
                        }
                    }
                )*
            }
        }
    }
}
