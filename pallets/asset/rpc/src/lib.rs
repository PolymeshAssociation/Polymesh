use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use polymesh_runtime_identity_rpc_runtime_api::{
    AssetDidResult, CddStatus, DidRecords, IdentityApi as IdentityRuntimeApi,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait CanTransferRpc {
    #[rpc(name = "asset_canTransfer")]
    fn can_transfer(
        &self, 
        ticker: Ticker, 
        from_did: IdentityId, 
        to_did: IdentityId, 
        value: T::Balance, 
        data: Vec<u8>
    ) -> Result<u8>;
}

pub struct CanTransferStruc;

impl CanTransferRpc for CanTransferStruc {
    fn can_transfer(
        &self, 
        ticker: Ticker, 
        from_did: IdentityId, 
        to_did: IdentityId, 
        value: T::Balance, 
        data: Vec<u8>
    ) -> Result<u8> {
            let api = self.client.runtime_api();
            let mut current_balance: T::Balance = self::balance(&ticker, &from_did);
            if current_balance < value {
                current_balance = 0.into();
            } else {
                current_balance -= value;
            }
            if current_balance < self::total_custody_allowance((ticker, from_did)) {
                sp_runtime::print("Insufficient balance");
                api.can_transfer(ticker, from_did, to_did, value, data, ERC1400_INSUFFICIENT_BALANCE as u32);
            } else {
                match self::_is_valid_transfer(&ticker, sender, Some(from_did), Some(to_did), value) {
                    Ok(code) =>
                    {
                        api.can_transfer(ticker, from_did, to_did, value, data, code as u32);
                    },
                    Err(msg) => {
                        // We emit a generic error with the event whenever there's an internal issue - i.e. captured
                        // in a string error and not using the status codes
                        sp_runtime::print(msg);
                        api.can_transfer(ticker, from_did, to_did, value, data, ERC1400_TRANSFER_FAILURE as u32);
                    }
                }
            }
    }
}







