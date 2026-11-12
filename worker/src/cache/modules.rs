use parking_lot::Mutex;
use std::{collections::BTreeMap, sync::Arc};

use polymesh_worker_common::*;

use crate::{
    StaticModules,
    backend::{BACKENDS, Backend, BackendModule, BackendModuleInstance, BackendModuleLoader},
    decompress_module_code,
};

pub const MAX_MODULE_INSTANCES_PER_PROTOCOL: usize = 32;

/// An instance of a loaded protocol module, which can be used to execute work requests.
///
/// Only instances whose execution completed without a worker error are returned to the cache.
pub struct ProtocolModuleInstance {
    instance: Option<Box<dyn BackendModuleInstance>>,
    module: ProtocolModuleRef,
}

impl Drop for ProtocolModuleInstance {
    fn drop(&mut self) {
        if let Some(instance) = self.instance.take() {
            self.module.return_instance(instance);
        }
    }
}

impl ProtocolModuleInstance {
    fn new(instance: Box<dyn BackendModuleInstance>, module: ProtocolModuleRef) -> Self {
        Self {
            instance: Some(instance),
            module,
        }
    }

    pub fn execute(&mut self, req: &WorkRequest) -> Result<WorkResponseResult, WorkerError> {
        let mut instance = self
            .instance
            .take()
            .ok_or(WorkerError::ModuleExecutionFailed)?;
        let result = instance.execute(req);
        if result.is_ok() {
            self.instance = Some(instance);
        }
        result
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::backend::Backends;
    use polymesh_worker_protocol_testing::{PROTOCOL, TestWorkRequest, VerifyVersionRequest};

    #[test]
    fn protocol_config_must_match_requested_protocol() {
        let requested = Protocol {
            id: PROTOCOL_PDART,
            version: ProtocolVersion::new(1, 0, 0),
        };
        let matching = ProtocolModuleConfig {
            protocol: requested,
            initialization_method: ProtocolInitializationMethod::NoInitializationNeeded,
            modules: Vec::new(),
        };
        assert!(protocol_matches(&matching, requested));

        let mismatched = ProtocolModuleConfig {
            protocol: Protocol {
                version: ProtocolVersion::new(0, 1, 0),
                ..requested
            },
            ..matching
        };
        assert!(!protocol_matches(&mismatched, requested));
    }

    #[test]
    fn trapped_instances_are_discarded_but_protocol_errors_are_cached() {
        let kinds = [
            #[cfg(all(feature = "polkavm", target_arch = "x86_64"))]
            BackendKind::PolkaVM,
            #[cfg(feature = "wasmer")]
            BackendKind::Wasmer,
            #[cfg(feature = "wasmtime")]
            BackendKind::Wasmtime,
        ];
        let backends = Backends::with_backends(&kinds, None);
        assert_eq!(backends.backends.len(), kinds.len());
        for backend in &backends.backends {
            let bytes: &[u8] = match backend.kind() {
                BackendKind::PolkaVM => include_bytes!(
                    "../../modules/testing/v0/polymesh-worker-protocol-testing.polkavm.zst"
                ),
                _ => include_bytes!(
                    "../../modules/testing/v0/polymesh-worker-protocol-testing.wasm.zst"
                ),
            };
            let module = ProtocolModuleRef(Arc::new(ProtocolModule {
                kind: backend.kind(),
                config_hash: [0; 32],
                code_hash: [0; 32],
                #[cfg(feature = "worker-differential")]
                expected_backends: vec![backend.kind()],
                module: backend
                    .load_module(&decompress_module_code(bytes).unwrap())
                    .unwrap(),
                initialization_method: ResolvedInitializationMethod::InitializeNoContext,
                cache: Mutex::new(ProtocolModuleInstanceCache::new()),
            }));
            {
                let mut instance = module.get_instance().unwrap();
                let request = TestWorkRequest::VerifyVersion(VerifyVersionRequest {
                    protocol: Protocol {
                        version: ProtocolVersion::new(99, 0, 0),
                        ..PROTOCOL
                    },
                });
                assert!(matches!(
                    instance.execute(&WorkRequest::new(request)),
                    Ok(Err(_))
                ));
            }
            assert_eq!(module.0.cache.lock().instances.len(), 1);
            {
                let mut instance = module.get_instance().unwrap();
                assert!(matches!(
                    instance.execute(&WorkRequest::new(TestWorkRequest::Trap)),
                    Err(WorkerError::ModuleExecutionFailed)
                ));
                assert!(instance.instance.is_none());
            }
            assert!(module.0.cache.lock().instances.is_empty());
            {
                let mut instance = module.get_instance().unwrap();
                let request =
                    TestWorkRequest::VerifyVersion(VerifyVersionRequest { protocol: PROTOCOL });
                assert!(matches!(
                    instance.execute(&WorkRequest::new(request)),
                    Ok(Ok(_))
                ));
            }
            assert_eq!(module.0.cache.lock().instances.len(), 1);
        }
    }
}

struct ProtocolModuleInstanceCache {
    instances: Vec<Box<dyn BackendModuleInstance>>,
}

impl ProtocolModuleInstanceCache {
    fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    fn get_instance(&mut self) -> Option<Box<dyn BackendModuleInstance>> {
        self.instances.pop()
    }

    fn return_instance(&mut self, instance: Box<dyn BackendModuleInstance>) {
        // If the cache is full, we simply drop the instance, which will free its resources.
        if self.instances.len() >= MAX_MODULE_INSTANCES_PER_PROTOCOL {
            return;
        }
        self.instances.push(instance);
    }
}

/// A loaded protocol module, which can be used to create module instances for executing work requests.
pub struct ProtocolModule {
    pub kind: BackendKind,
    pub config_hash: ProtocolModuleConfigHash,
    pub code_hash: BackendCodeHash,
    #[cfg(feature = "worker-differential")]
    expected_backends: Vec<BackendKind>,
    module: Box<dyn BackendModule>,
    initialization_method: ResolvedInitializationMethod,
    cache: Mutex<ProtocolModuleInstanceCache>,
}

impl ProtocolModule {
    fn load_from_backend(
        config: &ProtocolModuleConfig,
        config_hash: ProtocolModuleConfigHash,
        allowed: BackendBitmask,
        backend: &Box<dyn Backend>,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        #[cfg(not(feature = "worker-differential"))]
        let _ = allowed;

        let kind = backend.kind();
        let module_kind = kind.to_module_kind();

        // Find a compatible module definition for this backend in the protocol config.
        let mut found = None;
        for def in &config.modules {
            if def.module_kind == module_kind && backend.is_module_compatible(def.module_version) {
                found = Some(def);
                break;
            }
        }
        let module_def = found?;

        // Try getting the module bytes using the code hash.  If this fails, it means that the module code is not available for this backend.
        // This shouldn't happen if we got a code hash.
        let module_bytes =
            loader.get_module_code_bytes(protocol, module_kind, module_def.code_hash)?;

        // Decompress module bytes if it was compressed.
        let module_bytes = decompress_module_code(&module_bytes)?;

        // Try loading the module into the backend.  If this fails, it means that the module code is not compatible with this backend.
        let module = backend.load_module(&module_bytes)?;

        #[cfg(feature = "worker-differential")]
        let expected_backends = BACKENDS
            .read()
            .backends
            .iter()
            .filter(|candidate| {
                candidate.is_in_bitmask(allowed)
                    && config.modules.iter().any(|definition| {
                        definition.module_kind == candidate.kind().to_module_kind()
                            && candidate.is_module_compatible(definition.module_version)
                    })
            })
            .map(|candidate| candidate.kind())
            .collect();

        // Check if we should initialize the context from the first instance.  If so, we need to execute the initialization method once to get the context data, which will be cached for future instances.
        let initialization_method =
            if let ProtocolInitializationMethod::SaveContextFromFirstInstance =
                config.initialization_method
            {
                let mut instance = module.instantiate()?;
                if let Err(err) = instance.initialize(None) {
                    log::error!(
                        "Failed to initialize module instance for context caching: {err:?}"
                    );
                    return None;
                }
                match instance.save_context().ok()? {
                    Some(ctx_data) => {
                        log::info!(
                            "Successfully cached context data from first module instance for protocol: {:?}, backend: {:?}",
                            protocol,
                            backend.kind()
                        );
                        ResolvedInitializationMethod::ContextData(ctx_data)
                    }
                    None => {
                        log::warn!(
                            "Module instance does not support context saving, falling back to no context initialization for protocol: {:?}, backend: {:?}",
                            protocol,
                            backend.kind()
                        );
                        ResolvedInitializationMethod::InitializeNoContext
                    }
                }
            } else {
                log::info!(
                    "Using initialization method {:?} for protocol: {:?}, backend: {:?}",
                    config.initialization_method,
                    protocol,
                    backend.kind()
                );
                // If the initialization method is `ContextHash`, we need to load the context bytes.
                loader.resolve_initialization_method(protocol, &config.initialization_method)?
            };

        Some(ProtocolModuleRef(Arc::new(Self {
            kind,
            config_hash,
            code_hash: module_def.code_hash,
            #[cfg(feature = "worker-differential")]
            expected_backends,
            module,
            initialization_method,
            cache: Mutex::new(ProtocolModuleInstanceCache::new()),
        })))
    }
}

/// A shared reference to a loaded protocol module, which can be used to create module instances for executing work requests.
#[derive(Clone)]
pub struct ProtocolModuleRef(Arc<ProtocolModule>);

impl ProtocolModuleRef {
    /// Get a module instance from the cache or create a new one if the cache is empty.
    pub fn get_instance(&self) -> Option<ProtocolModuleInstance> {
        // First try getting an instance from the cache.
        {
            let mut cache = self.0.cache.lock();
            if let Some(instance) = cache.get_instance() {
                log::debug!(
                    "Reusing cached module instance for protocol: {:?}",
                    self.0.kind
                );
                return Some(ProtocolModuleInstance::new(instance, self.clone()));
            }
        }

        log::info!(
            "No cached module instance available, creating a new one for protocol: {:?}",
            self.0.kind
        );
        // No cached instance available, create a new one.
        let mut instance = self.0.module.instantiate()?;

        // Initialize the module instance if needed.
        match &self.0.initialization_method {
            ResolvedInitializationMethod::NoInitializationNeeded => {
                // No initialization needed, do nothing.
            }
            ResolvedInitializationMethod::InitializeNoContext => {
                if let Err(err) = instance.initialize(None) {
                    log::error!("Failed to initialize module instance: {err:?}");
                    return None;
                }
            }
            ResolvedInitializationMethod::ContextData(ctx_data) => {
                if let Err(err) = instance.initialize(Some(ctx_data)) {
                    log::error!("Failed to initialize module instance: {err:?}");
                    return None;
                }
            }
        }

        Some(ProtocolModuleInstance::new(instance, self.clone()))
    }

    /// Return a module instance to the cache for future reuse.
    pub fn return_instance(&self, instance: Box<dyn BackendModuleInstance>) {
        let mut cache = self.0.cache.lock();
        cache.return_instance(instance);
    }

    /// Get the code hash of the module.
    pub fn code_hash(&self) -> BackendCodeHash {
        self.0.code_hash
    }

    /// Get the hash of the protocol config (code hashes + initialization context) the module was loaded from.
    pub fn config_hash(&self) -> ProtocolModuleConfigHash {
        self.0.config_hash
    }

    /// Get the backend kind used by this protocol module.
    pub fn kind(&self) -> BackendKind {
        self.0.kind
    }

    /// Get backends with compatible modules declared by this protocol config.
    #[cfg(feature = "worker-differential")]
    pub fn expected_backends(&self) -> &[BackendKind] {
        &self.0.expected_backends
    }

    /// Fingerprint the context loaded by this backend, if one is configured.
    pub fn context_hash(&self) -> Option<BackendContextHash> {
        match &self.0.initialization_method {
            ResolvedInitializationMethod::ContextData(context) => {
                Some(crate::blake2b256_hash(context))
            }
            ResolvedInitializationMethod::NoInitializationNeeded
            | ResolvedInitializationMethod::InitializeNoContext => None,
        }
    }
}

/// Check that a loaded protocol config belongs to the requested protocol.
///
/// On-chain protocol configs are looked up by protocol, but a malicious or buggy config could
/// contain a different protocol.  Reject it so the caller falls back to the builtin static
/// modules, which are keyed by the exact protocol (id and version).
fn protocol_matches(config: &ProtocolModuleConfig, protocol: Protocol) -> bool {
    if config.protocol == protocol {
        true
    } else {
        log::error!(
            "Protocol module config mismatch: requested {:?}, got {:?}",
            protocol,
            config.protocol
        );
        false
    }
}

pub(crate) struct BackendModuleCache {
    protocols: BTreeMap<(ProtocolModuleConfigHash, BackendKind, BackendBitmask), ProtocolModuleRef>,
    builtin: StaticModules,
}

impl BackendModuleCache {
    pub(crate) fn new() -> Self {
        let static_module = StaticModules::new();
        Self {
            protocols: BTreeMap::new(),
            builtin: static_module,
        }
    }

    pub(crate) fn load_protocol(
        &mut self,
        allowed: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        // First try loading from `loader`.
        if let Some(config_hash) = loader.get_protocol_module_config_hash(protocol) {
            if let Some(module) = Self::load_first_from_config(
                &mut self.protocols,
                allowed,
                protocol,
                config_hash,
                loader,
            ) {
                return Some(module);
            }
        }

        // Fallback to the builtin static modules.
        let config_hash = self.builtin.get_protocol_module_config_hash(protocol)?;
        Self::load_first_from_config(
            &mut self.protocols,
            allowed,
            protocol,
            config_hash,
            &mut self.builtin,
        )
    }

    pub(crate) fn load_protocols(
        &mut self,
        allowed: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Vec<ProtocolModuleRef> {
        // First try loading from `loader`.
        if let Some(config_hash) = loader.get_protocol_module_config_hash(protocol) {
            let loaded = Self::load_all_from_config(
                &mut self.protocols,
                allowed,
                protocol,
                config_hash,
                loader,
            );
            if !loaded.is_empty() {
                return loaded;
            }
        }

        // Fallback to the builtin static modules.
        if let Some(config_hash) = self.builtin.get_protocol_module_config_hash(protocol) {
            return Self::load_all_from_config(
                &mut self.protocols,
                allowed,
                protocol,
                config_hash,
                &mut self.builtin,
            );
        }

        Vec::new()
    }

    fn load_first_from_config(
        protocols: &mut BTreeMap<
            (ProtocolModuleConfigHash, BackendKind, BackendBitmask),
            ProtocolModuleRef,
        >,
        allowed: BackendBitmask,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        let backends = BACKENDS.read();
        let mut config = None;

        for backend in &backends.backends {
            if !backend.is_in_bitmask(allowed) {
                continue;
            }

            let kind = backend.kind();
            // The allowed mask is part of the cache identity: the module's
            // `expected_backends` (used by the differential log) must reflect
            // the mask of this load, not the mask of the first load.
            let key = (config_hash, kind, allowed);
            if let Some(module) = protocols.get(&key) {
                return Some(module.clone());
            }

            if config.is_none() {
                config = loader.get_protocol_module_config(protocol, config_hash);
            }
            let Some(config) = config.as_ref() else {
                return None;
            };
            if !protocol_matches(config, protocol) {
                return None;
            }

            if let Some(module) = ProtocolModule::load_from_backend(
                config,
                config_hash,
                allowed,
                backend,
                protocol,
                loader,
            ) {
                log::info!("Loaded protocol module using backend {kind:?}");
                protocols.insert(key, module.clone());
                return Some(module);
            }
        }
        None
    }

    fn load_all_from_config(
        protocols: &mut BTreeMap<
            (ProtocolModuleConfigHash, BackendKind, BackendBitmask),
            ProtocolModuleRef,
        >,
        allowed: BackendBitmask,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
        loader: &mut dyn BackendModuleLoader,
    ) -> Vec<ProtocolModuleRef> {
        let backends = BACKENDS.read();
        let mut modules = Vec::new();
        let mut config = None;
        for backend in &backends.backends {
            if !backend.is_in_bitmask(allowed) {
                continue;
            }

            let kind = backend.kind();
            // The allowed mask is part of the cache identity: the module's
            // `expected_backends` (used by the differential log) must reflect
            // the mask of this load, not the mask of the first load.
            let key = (config_hash, kind, allowed);
            if let Some(module) = protocols.get(&key) {
                modules.push(module.clone());
                continue;
            }

            if config.is_none() {
                config = loader.get_protocol_module_config(protocol, config_hash);
            }
            let Some(config) = config.as_ref() else {
                return modules;
            };
            if !protocol_matches(config, protocol) {
                return modules;
            }

            if let Some(module) = ProtocolModule::load_from_backend(
                config,
                config_hash,
                allowed,
                backend,
                protocol,
                loader,
            ) {
                log::info!("Loaded protocol module using backend {kind:?}");
                protocols.insert(key, module.clone());
                modules.push(module);
            }
        }
        modules
    }
}
