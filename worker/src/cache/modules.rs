use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use polymesh_worker_common::*;

use crate::{
    StaticModules,
    backend::{BACKENDS, Backend, BackendModule, BackendModuleInstance, BackendModuleLoader},
};

pub const MAX_MODULE_INSTANCES_PER_PROTOCOL: usize = 32;

/// An instance of a loaded protocol module, which can be used to execute work requests.
///
/// The instance will be returned to the module's cache when dropped, so it can be reused for future requests.
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
        if let Some(instance) = &mut self.instance {
            instance.execute(req)
        } else {
            // This shouldn't happen because the instance should only be taken when dropping, but we return an error just in case.
            Err(WorkerError::ModuleExecutionFailed)
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
    pub code_hash: BackendCodeHash,
    module: Box<dyn BackendModule>,
    initialization_method: ResolvedInitializationMethod,
    cache: RwLock<ProtocolModuleInstanceCache>,
}

impl ProtocolModule {
    fn load_from_backend(
        config: &ProtocolModuleConfig,
        backend: &Box<dyn Backend>,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
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

        // Try loading the module into the backend.  If this fails, it means that the module code is not compatible with this backend.
        let module = backend.load_module(&module_bytes)?;

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
            code_hash: module_def.code_hash,
            module,
            initialization_method,
            cache: RwLock::new(ProtocolModuleInstanceCache::new()),
        })))
    }

    pub fn load(
        allowed: BackendBitmask,
        protocol: Protocol,
        config_hash: ProtocolModuleConfigHash,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        let backends = BACKENDS.read().unwrap();

        // Load protocol config.
        let config = loader.get_protocol_module_config(protocol, config_hash)?;

        for backend in &backends.backends {
            if !backend.is_in_bitmask(allowed) {
                continue;
            }
            let module = Self::load_from_backend(&config, backend, protocol, loader);
            if module.is_some() {
                log::info!("Loaded protocol module using backend {:?}", backend.kind());
                return module;
            }
        }
        None
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
            let mut cache = self.0.cache.write().unwrap();
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
        let mut cache = self.0.cache.write().unwrap();
        cache.return_instance(instance);
    }

    /// Get the code hash of the module, which is used for caching work responses.
    pub fn code_hash(&self) -> BackendCodeHash {
        self.0.code_hash
    }
}

pub(crate) struct BackendModuleCache {
    protocols: BTreeMap<ProtocolModuleConfigHash, ProtocolModuleRef>,
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
            if let Some(module) = self.protocols.get(&config_hash) {
                return Some(module.clone());
            }

            // Try loading the protocol module from the available backends.
            if let Some(module) = ProtocolModule::load(allowed, protocol, config_hash, loader) {
                self.protocols.insert(config_hash, module.clone());
                return Some(module);
            }
        }

        // Fallback to the builtin static modules.
        if let Some(config_hash) = self.builtin.get_protocol_module_config_hash(protocol) {
            if let Some(module) = self.protocols.get(&config_hash) {
                return Some(module.clone());
            }

            if let Some(module) =
                ProtocolModule::load(allowed, protocol, config_hash, &mut self.builtin)
            {
                self.protocols.insert(config_hash, module.clone());
                return Some(module);
            }
        }

        None
    }
}
