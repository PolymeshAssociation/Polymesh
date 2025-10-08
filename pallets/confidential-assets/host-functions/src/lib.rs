// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh Association

#![cfg_attr(not(feature = "std"), no_std)]

mod verify;
pub use verify::*;

mod curve_tree;
pub use curve_tree::*;

mod asset;
pub use asset::*;

#[cfg(feature = "std")]
pub mod batch;

#[cfg(feature = "std")]
mod cache;

#[cfg(feature = "testing")]
mod testing;

#[cfg(not(feature = "testing"))]
mod testing {
    use codec::{Decode, Encode};

    /// The non-testing version of this enum is empty, as no requests are currently supported
    #[derive(Encode, Decode, Clone)]
    pub enum GenerateDartProofRequest {}

    /// The non-testing version of this enum is empty, as no requests are currently supported
    #[derive(Encode, Decode, Clone)]
    pub enum GenerateDartProofResponse {}
}

pub use testing::*;

use codec::{Decode, Encode};
use sp_io::hashing::blake2_256;
use sp_runtime_interface::{pass_by::*, runtime_interface};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::Vec;

use polymesh_dart::{
    ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M, FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
    curve_tree::{
        AccountTreeConfig, AssetTreeConfig, CompressedCurveTreeRoot, FeeAccountTreeConfig,
    },
};

pub const MIN_SUPPORTED_VERSION: WorkRequestVersion = 0;
pub const MAX_SUPPORTED_VERSION: WorkRequestVersion = 1;
pub const CURRENT_VERSION: WorkRequestVersion = 1;

pub type BatchId = u32;

pub type BatchSeed = [u8; 32];

pub type WorkRequestId = u32;

/// A hash of a request.
pub type WorkRequestHash = [u8; 32];
/// The request version number is used to force the runtime to fallback to verifying
/// the proof in `no_std` environment if the current host node does not support the request version.
pub type WorkRequestVersion = u32;

/// The kind of a batch request.
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub enum WorkRequestKind {
    VerifyProof(#[codec(compact)] WorkRequestVersion),
    GenerateProof(#[codec(compact)] WorkRequestVersion),
}

impl WorkRequestKind {
    /// Check if the request can use a cached value.
    /// Proof generation requests can always use a cached value.
    /// Proof verification requests can use a cached value unless we are in testing mode,
    /// where we want to benchmark the proof verification.
    pub fn use_cached_value(&self) -> bool {
        match self {
            WorkRequestKind::VerifyProof(_) => {
                #[cfg(feature = "testing")]
                {
                    // In testing we need to benchmark the proof verification, so we disable using the cached value.
                    // We still need to benchmark the overhead of using the cache (checking the cache and saving results to the cache).
                    false
                }
                #[cfg(not(feature = "testing"))]
                {
                    true
                }
            }
            WorkRequestKind::GenerateProof(_) => true,
        }
    }
}

/// Where to execute a work request.
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub enum ExecutionKind {
    HostPool,
    Host,
    Runtime,
}

impl ExecutionKind {
    /// Check if the request should be executed in the host pool.
    pub fn host_pool(&self) -> bool {
        matches!(self, ExecutionKind::HostPool)
    }

    /// Check if the request must be executed on the runtime side.
    pub fn runtime(&self) -> bool {
        matches!(self, ExecutionKind::Runtime)
    }
}

/// Where to execute a batch request.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkRequestExecution {
    pub req_id: WorkRequestId,
    pub kind: ExecutionKind,
    pub req_hash: WorkRequestHash,
}

impl WorkRequestExecution {
    /// Check if the request should be executed in the host pool.
    pub fn host_pool(&self) -> bool {
        self.kind.host_pool()
    }

    /// Check if the request must be executed on the runtime side.
    pub fn runtime_side(&self) -> bool {
        self.kind.runtime()
    }

    pub fn wait_for_results(&self, batch_id: BatchId) -> Result<WorkResponse, Error> {
        #[cfg(feature = "std")]
        {
            batch::BatchVerifiers::get_or_wait_for(batch_id, self.req_id)?
        }
        #[cfg(not(feature = "std"))]
        {
            native_dart_assets::batch_get_or_wait_for_result(batch_id, self.req_id)?
        }
    }
}

/// A batch request.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkRequest {
    pub kind: WorkRequestKind,
    pub req: Vec<u8>,
}

impl WorkRequest {
    /// Check if the request is supported by the current host node.
    pub fn is_supported(&self) -> bool {
        match &self.kind {
            WorkRequestKind::VerifyProof(version) | WorkRequestKind::GenerateProof(version) => {
                *version <= MAX_SUPPORTED_VERSION
            }
        }
    }

    /// Create a new batch request for verifying a proof.
    pub fn new_verify(req: &VerifyDartAssetRequest) -> Self {
        Self {
            kind: WorkRequestKind::VerifyProof(CURRENT_VERSION),
            req: req.encode(),
        }
    }

    /// Create a new batch request for generating a proof.
    pub fn new_generate(req: &GenerateDartProofRequest) -> Self {
        Self {
            kind: WorkRequestKind::GenerateProof(CURRENT_VERSION),
            req: req.encode(),
        }
    }

    /// Where to execute the request.
    pub fn execution(&self, req_id: WorkRequestId, use_pool: bool) -> WorkRequestExecution {
        let req_hash = blake2_256(&self.req);
        let kind = match (self.is_supported(), use_pool) {
            (true, true) => ExecutionKind::HostPool,
            (true, false) => ExecutionKind::Host,
            (false, _) => ExecutionKind::Runtime,
        };
        WorkRequestExecution {
            req_id,
            kind,
            req_hash,
        }
    }

    pub fn decode<T: Decode>(&self) -> Result<T, Error> {
        T::decode(&mut &self.req[..]).map_err(|_| Error::DecodingFailed)
    }

    /// Decode and execute the batch request.
    pub fn execute(&self, hash: WorkRequestHash) -> Result<WorkResponse, Error> {
        match self.kind {
            WorkRequestKind::VerifyProof(_) => {
                let req: VerifyDartAssetRequest = self.decode()?;
                let resp = req.verify_with_seed(hash)?;
                Ok(WorkResponse {
                    kind: self.kind,
                    resp: resp.encode(),
                })
            }
            WorkRequestKind::GenerateProof(_) => {
                #[cfg(feature = "testing")]
                {
                    let req: GenerateDartProofRequest = self.decode()?;
                    let proof = req.generate_with_seed(hash)?;
                    Ok(WorkResponse {
                        kind: self.kind,
                        resp: proof.encode(),
                    })
                }
                #[cfg(not(feature = "testing"))]
                Err(Error::GenerateProofFailed)
            }
        }
    }

    /// Skip proof verification and return a successful response.
    /// Only available for benchmarking to isolate the proof verification
    #[cfg(feature = "testing")]
    pub fn skip_verify(&self) -> Result<WorkResponse, Error> {
        match self.kind {
            WorkRequestKind::VerifyProof(_) => {
                let req: VerifyDartAssetRequest = self.decode()?;
                let resp = req.get_response()?;
                Ok(WorkResponse {
                    kind: self.kind,
                    resp: resp.encode(),
                })
            }
            WorkRequestKind::GenerateProof(_) => Err(Error::GenerateProofFailed),
        }
    }

    /// Submit the request to a batch for execution.
    pub fn submit(self, id: BatchId) -> Result<WorkRequestExecution, Error> {
        #[cfg(feature = "std")]
        {
            batch::BatchVerifiers::submit(id, self)
        }
        #[cfg(not(feature = "std"))]
        {
            let execution = native_dart_assets::batch_submit(id, self.clone())?;
            // check for fall back to runtime execution.
            if execution.runtime_side() {
                // Execute the work request in the runtime (WASM).
                let result = self.execute(execution.req_hash);
                // Push the result back to the batch.
                native_dart_assets::batch_push_result(id, &execution, result)?;
            }
            Ok(execution)
        }
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkRequestResult {
    pub id: WorkRequestId,
    pub result: Result<WorkResponse, Error>,
}

impl WorkRequestResult {
    pub fn decode<T: Decode>(&self) -> Result<T, Error> {
        match &self.result {
            Ok(resp) => resp.decode(),
            Err(err) => Err(err.clone()),
        }
    }
}

/// A batch response.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkResponse {
    pub kind: WorkRequestKind,
    pub resp: Vec<u8>,
}

impl WorkResponse {
    /// Decode the response.
    pub fn decode<T: Decode>(&self) -> Result<T, Error> {
        T::decode(&mut &self.resp[..]).map_err(|_| Error::DecodingFailed)
    }
}

pub type AssetTreeRoot = CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;
pub type AccountTreeRoot =
    CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;
pub type FeeAccountTreeRoot =
    CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    VerifyFailed,
    BatchClosed,
    GenerateProofFailed,
    DecodingFailed,
    MissingBatch,
    CurveTreeUpdateError,
    AssetStateError,
    InvalidWorkResult,
}

impl From<polymesh_dart::Error> for Error {
    fn from(_e: polymesh_dart::Error) -> Self {
        Self::VerifyFailed
    }
}

/// Native interface for runtime module for DART Assets
#[runtime_interface]
pub trait NativeDARTAssets {
    fn verify_proof(request: PassFatPointerAndDecode<VerifyDartAssetRequest>) -> AllocateAndReturnByCodec<Result<VerifyDartProofResponse, Error>> {
        request.verify()
    }

    fn generate_proof(
        _request: PassFatPointerAndDecode<GenerateDartProofRequest>,
    ) -> AllocateAndReturnByCodec<Result<GenerateDartProofResponse, Error>> {
        #[cfg(feature = "testing")]
        {
            _request.generate()
        }
        #[cfg(not(feature = "testing"))]
        Err(Error::GenerateProofFailed)
    }

    fn create_batch(use_cache: bool, use_pool: bool) -> BatchId {
        batch::BatchVerifiers::create_batch(use_cache, use_pool)
    }

    fn batch_use_thread_pool(id: BatchId, use_pool: bool) -> AllocateAndReturnByCodec<Result<(), Error>> {
        batch::BatchVerifiers::use_thread_pool(id, use_pool)
    }

    fn batch_use_cache(id: BatchId, use_cache: bool) -> AllocateAndReturnByCodec<Result<(), Error>> {
        batch::BatchVerifiers::use_cache(id, use_cache)
    }

    fn batch_submit(id: BatchId, req: PassFatPointerAndDecode<WorkRequest>) -> AllocateAndReturnByCodec<Result<WorkRequestExecution, Error>> {
        batch::BatchVerifiers::submit(id, req)
    }

    fn batch_push_result(
        id: BatchId,
        execution: PassFatPointerAndDecode<WorkRequestExecution>,
        result: PassFatPointerAndDecode<Result<WorkResponse, Error>>,
    ) -> AllocateAndReturnByCodec<Result<(), Error>> {
        batch::BatchVerifiers::push_result(id, &execution, result)
    }

    fn batch_get_or_wait_for_result(
        id: BatchId,
        wait_for: WorkRequestId,
    ) -> AllocateAndReturnByCodec<Result<Result<WorkResponse, Error>, Error>> {
        batch::BatchVerifiers::get_or_wait_for(id, wait_for)
    }

    fn batch_next_result(
        id: BatchId,
    ) -> AllocateAndReturnByCodec<Result<Option<(WorkRequestId, Result<WorkResponse, Error>)>, Error>> {
        batch::BatchVerifiers::next_result(id)
    }

    fn batch_finish(
        id: BatchId,
    ) -> AllocateAndReturnByCodec<Result<BTreeMap<WorkRequestId, Result<WorkResponse, Error>>, Error>> {
        let batch = batch::BatchVerifiers::finish(id).ok_or(Error::VerifyFailed)?;
        batch.finalize()
    }

    fn batch_close(id: BatchId) {
        batch::BatchVerifiers::close(id);
    }

    fn set_skip_verify(_skip: bool) {
        #[cfg(feature = "testing")]
        batch::BatchVerifiers::set_skip_verify(_skip);
    }

    fn asset_tree_update_inner_node(
        req: PassFatPointerAndDecode<UpdateTreeNodeRequest<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>>,
    ) -> AllocateAndReturnByCodec<Result<UpdateTreeNodeResult<ASSET_TREE_M>, Error>> {
        req.update()
    }

    fn account_tree_update_inner_node(
        req: PassFatPointerAndDecode<UpdateTreeNodeRequest<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>>,
    ) -> AllocateAndReturnByCodec<Result<UpdateTreeNodeResult<ACCOUNT_TREE_M>, Error>> {
        req.update()
    }

    fn fee_account_tree_update_inner_node(
        req: PassFatPointerAndDecode<UpdateTreeNodeRequest<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>>,
    ) -> AllocateAndReturnByCodec<Result<UpdateTreeNodeResult<FEE_ACCOUNT_TREE_M>, Error>> {
        req.update()
    }

    fn update_asset_state(req: PassFatPointerAndDecode<UpdateAssetStateRequest>) -> AllocateAndReturnByCodec<Result<UpdateAssetStateResult, Error>> {
        req.update()
    }
}
