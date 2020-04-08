//! Runtime API definition for Identity module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use polymesh_primitives::IdentityId;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{prelude::*, vec::Vec};


sp_api::decl_runtime_apis! {
    
}
