use anyhow::Result;

use polymesh_api::{types::polymesh_worker_common::BackendModuleDefinition, Api};
use polymesh_api_tester::{DbAccountSigner, PolymeshTester};

use codec::Encode;
pub use polymesh_api::types::{
    pallet_worker_modules::*,
    polymesh_worker_common::{BackendModuleKind, Protocol, ProtocolId, ProtocolVersion},
    runtime::RuntimeCall,
};
use sp_core::blake2_256;

pub struct WorkerModulesHelper {
    pub api: Api,
    pub protocol: Protocol,
    sudo: DbAccountSigner,
}

impl WorkerModulesHelper {
    pub fn new(tester: &PolymeshTester, protocol: Protocol) -> Self {
        let api = tester.api.clone();
        let sudo = tester
            .sudo
            .as_ref()
            .expect("Sudo account not found")
            .clone();
        Self {
            api,
            protocol,
            sudo,
        }
    }

    pub fn update_version(&mut self, version: ProtocolVersion) {
        self.protocol.version = version;
    }

    pub async fn sudo_call<C: Into<RuntimeCall>>(&mut self, call: C) -> Result<()> {
        let mut res = self
            .api
            .call()
            .sudo()
            .sudo(call.into())?
            .submit_and_watch(&mut self.sudo)
            .await?;
        res.wait_finalized().await?;
        Ok(())
    }

    pub async fn register_protocol(
        &mut self,
        name: &str,
        description: &str,
        version: ProtocolVersion,
    ) -> Result<()> {
        self.sudo_call(self.api.call().worker_modules().register_protocol(
            self.protocol.id.clone(),
            ProtocolMetadata {
                protocol_name: name.as_bytes().to_vec(),
                protocol_description: description.as_bytes().to_vec(),
                protocol_version: version,
            },
        )?)
        .await?;

        Ok(())
    }

    pub async fn upload_module_code(&mut self, module_code: Vec<u8>) -> Result<()> {
        self.sudo_call(
            self.api
                .call()
                .worker_modules()
                .upload_protocol_module_code(self.protocol.clone(), module_code)?,
        )
        .await?;

        Ok(())
    }

    pub async fn upload_module_context(&mut self, module_context: Vec<u8>) -> Result<()> {
        self.sudo_call(
            self.api
                .call()
                .worker_modules()
                .upload_protocol_module_context(self.protocol.clone(), module_context)?,
        )
        .await?;

        Ok(())
    }

    pub async fn upload_config(
        &mut self,
        init_method: ProtocolInitializationMethod,
        modules: Vec<BackendModuleDefinition>,
    ) -> Result<()> {
        let config = ProtocolModuleConfig {
            protocol: self.protocol.clone(),
            initialization_method: init_method.clone(),
            modules: modules.clone(),
        };
        self.sudo_call(
            self.api
                .call()
                .worker_modules()
                .upload_protocol_module_config(config)?,
        )
        .await?;

        Ok(())
    }

    pub async fn upload_modules_and_config(
        &mut self,
        init_method: ProtocolInitializationMethod,
        modules: Vec<(BackendModuleKind, u32, Vec<u8>)>,
    ) -> Result<()> {
        let mut module_defs = Vec::new();
        for (module_kind, module_version, code) in modules.into_iter() {
            let code_hash = code.using_encoded(blake2_256);
            self.upload_module_code(code).await?;
            module_defs.push(BackendModuleDefinition {
                module_kind,
                module_version: module_version,
                code_hash,
            });
        }

        self.upload_config(init_method, module_defs).await?;

        Ok(())
    }
}
