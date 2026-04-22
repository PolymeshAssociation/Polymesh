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
    context: Option<Vec<u8>>,
    cache: RwLock<ProtocolModuleInstanceCache>,
}

impl ProtocolModule {
    fn load_from_backend(
        backend: &Box<dyn Backend>,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        let kind = backend.kind();

        // Get the module code hash for the given protocol, version and backend kind.  If the loader returns `None`, it means that the module is not available for this backend.
        let hash = loader.get_module_code_and_context_hash(protocol, kind)?;

        // Try getting the module bytes using the code hash.  If this fails, it means that the module code is not available for this backend.
        // This shouldn't happen if we got a code hash.
        let module_bytes = loader.get_module_code_bytes(protocol, kind, hash.code_hash)?;

        // Try loading the module into the backend.  If this fails, it means that the module code is not compatible with this backend.
        let module = backend.load_module(&module_bytes)?;

        // If there is a context hash, try loading the context.
        let context = if let Some(ctx_hash) = hash.context_hash {
            loader.get_module_context_bytes(protocol, ctx_hash)
        } else {
            None
        };

        Some(ProtocolModuleRef(Arc::new(Self {
            kind,
            code_hash: hash.code_hash,
            module,
            context,
            cache: RwLock::new(ProtocolModuleInstanceCache::new()),
        })))
    }

    pub fn load(
        allowed: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        let backends = BACKENDS.read().unwrap();
        for backend in &backends.backends {
            if !backend.is_in_bitmask(allowed) {
                continue;
            }
            let module = Self::load_from_backend(backend, protocol, loader);
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
        if let Err(err) = instance.initialize(self.0.context.as_deref()) {
            log::error!("Failed to initialize module instance: {err:?}");
            return None;
        }
        Some(ProtocolModuleInstance::new(instance, self.clone()))
    }

    /// Return a module instance to the cache for future reuse.
    pub fn return_instance(&self, instance: Box<dyn BackendModuleInstance>) {
        let mut cache = self.0.cache.write().unwrap();
        cache.return_instance(instance);
    }
}

pub(crate) struct BackendModuleCache {
    // TODO: change key to (Protocol, BackendKind) if we want to support multiple backends for the same protocol version.
    protocols: BTreeMap<Protocol, ProtocolModuleRef>,
}

impl BackendModuleCache {
    pub(crate) fn new() -> Self {
        Self {
            protocols: BTreeMap::new(),
        }
    }

    pub(crate) fn load_protocol(
        &mut self,
        allowed: BackendBitmask,
        protocol: Protocol,
        loader: &mut dyn BackendModuleLoader,
    ) -> Option<ProtocolModuleRef> {
        if let Some(module) = self.protocols.get(&protocol) {
            return Some(module.clone());
        }
        // Try loading the protocol module from the available backends.
        if let Some(module) = ProtocolModule::load(allowed, protocol, loader) {
            self.protocols.insert(protocol, module.clone());
            return Some(module);
        }
        // Fallback to the static modules.
        let mut static_module = StaticModules::new();
        if let Some(module) = ProtocolModule::load(allowed, protocol, &mut static_module) {
            self.protocols.insert(protocol, module.clone());
            return Some(module);
        }

        None
    }

    pub(crate) fn get_protocol(&self, protocol: Protocol) -> Option<ProtocolModuleRef> {
        self.protocols.get(&protocol).cloned()
    }

    pub(crate) fn get_protocol_instance(
        &self,
        protocol: Protocol,
    ) -> Option<ProtocolModuleInstance> {
        self.get_protocol(protocol)?.get_instance()
    }
}
