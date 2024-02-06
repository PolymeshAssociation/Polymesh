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
                        $(,)?
                        $($param:ident: $ty:ty),*
                        $(,)?
                    ) -> $fn_return:ty {
                        $( $fn_impl:tt )*
                    }
                )*
            }
            $(
            impl $api_type2:ident {
                $(
                    $(#[doc = $doc2_attr:tt])*
                    $fn2_vis:vis fn $fn2_name:ident(
                        $(& $self2:ident)?
                        $(,)?
                        $($param2:ident: $ty2:ty),*
                        $(,)?
                    ) -> $fn2_return:ty {
                        $( $fn2_impl:tt )*
                    }
                )*
            }
            )?
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
                    $(#[ink($ink_attr, payable)])*
                    $fn_vis fn $fn_name(&self, $($param: $ty),*) -> $fn_return {
                        ::paste::paste! {
                            self.[<__impl_ $fn_name>]($($param),*)
                        }
                    }
                )*
            }

            impl $api_type {
                $(
                    ::paste::paste! {
                        fn [<__impl_ $fn_name>](&$self, $($param: $ty),*) -> $fn_return {
                            $( $fn_impl )*
                        }
                    }
                )*
            }
            $(
            impl $api_type {
                $(
                    $(#[doc = $doc2_attr])*
                    $fn2_vis fn $fn2_name($(&$self2,)? $($param2: $ty2),*) -> $fn2_return {
                        $( $fn2_impl )*
                    }
                )*
            }
            )?
        }

        #[cfg(feature = "as-library")]
        mod $mod_name {
            use super::*;

            /// Upgradable wrapper for the Polymesh Runtime API.
            ///
            /// Contracts can use this to maintain support accross
            /// major Polymesh releases.
            #[derive(Clone, Debug, Default, scale::Encode, scale::Decode)]
            #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
            pub struct $api_type {
                hash: Hash,
            }

            impl $api_type {
                pub fn new() -> PolymeshResult<Self> {
                    Ok(Self {
                      hash: Self::get_latest_upgrade()?,
                    })
                }

                pub fn new_with_hash(hash: Hash) -> Self {
                    Self { hash }
                }

                /// Update code hash.
                pub fn update_code_hash(&mut self, hash: Hash) {
                    self.hash = hash;
                }

                pub fn check_for_upgrade(&mut self) -> PolymeshResult<()> {
                    self.hash = Self::get_latest_upgrade()?;
                    Ok(())
                }

                fn get_latest_upgrade() -> PolymeshResult<Hash> {
                    let extension = <<PolymeshEnvironment as ink::env::Environment>::ChainExtension as ink::ChainExtensionInstance>::instantiate();
                    Ok(extension.get_latest_api_upgrade((&API_VERSION).into())?.into())
                }

                $(
                    $crate::upgradable_api! {
                        @impl_api_func
                        $(#[doc = $doc_attr])*
                        $(#[ink($ink_attr)])*
                        $fn_vis fn $fn_name(
                            &$self,
                            $($param: $ty),*
                        ) -> $fn_return {
                            $( $fn_impl )*
                        }
                    }
                )*
            }
            $(
            impl $api_type {
                $(
                    $(#[doc = $doc2_attr])*
                    $fn2_vis fn $fn2_name($(&$self2,)? $($param2: $ty2),*) -> $fn2_return {
                        $( $fn2_impl )*
                    }
                )*
            }
            )?
        }
    };
    // Upgradable api method.
    (@impl_api_func
        $(#[doc = $doc_attr:tt])*
        $(#[ink($ink_attr:tt)])+
        $fn_vis:vis fn $fn_name:ident(
            & $self:ident
            $(,)?
            $($param:ident: $ty:ty),*
            $(,)?
        ) -> $fn_return:ty {
            $( $fn_impl:tt )*
        }
    ) => {
        $(#[doc = $doc_attr])*
        $fn_vis fn $fn_name(&$self, $($param: $ty),*) -> $fn_return {
            use ink::env::call::{ExecutionInput, Selector};
            const FUNC: &'static str = stringify!{$fn_name};
            let selector = Selector::new(::polymesh_api::ink::blake2_256(FUNC.as_bytes())[..4]
              .try_into().unwrap());
            ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                .delegate($self.hash)
                .exec_input(
                    ExecutionInput::new(selector)
                        .push_arg(($($param),*)),
                )
                .returns::<$fn_return>()
                .invoke()
        }
    };
    // Non-upgradable api method.
    (@impl_api_func
        $(#[doc = $doc_attr:tt])*
        $fn_vis:vis fn $fn_name:ident(
            $(& $self:ident)?
            $(,)?
            $($param:ident: $ty:ty),*
            $(,)?
        ) -> $fn_return:ty {
            $( $fn_impl:tt )*
        }
    ) => {
        $(#[doc = $doc_attr])*
        $fn_vis fn $fn_name($(&$self,)? $($param: $ty),*) -> $fn_return {
            $( $fn_impl )*
        }
    };
}
