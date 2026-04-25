use ark_host_msm_impl::host_msm_unchecked;
use bulletproofs::batch_hash_to_curve;
use sp_wasm_interface::*;

#[derive(Clone, Copy)]
pub struct HostFn {
    name: &'static str,
    func: fn(&mut [u8], u32) -> u32,
}

impl HostFn {
    fn call(&self, context: &mut dyn FunctionContext, fat_ptr: u64) -> Result<u32> {
        let (ptr, len) = ark_host_msm_impl::unpack_fat_pointer(fat_ptr);
        let ptr = Pointer::new(ptr);

        let mut buffer = context.read_memory(ptr, len)?;
        let res_len = (self.func)(buffer.as_mut_slice(), len);
        if res_len > 0 {
            context.write_memory(ptr, &buffer[..res_len as usize])?;
        }
        Ok(res_len)
    }

    fn protected_call<T>(
        &self,
        caller: wasmtime::Caller<T::State>,
        fat_ptr: u64,
    ) -> wasmtime::Result<u32>
    where
        T: HostFunctionRegistry,
    {
        T::with_function_context(caller, move |context| {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.call(context, fat_ptr)
                    .map_err(::sp_wasm_interface::anyhow::Error::msg)
            }));
            match result {
                Ok(result) => result,
                Err(panic) => {
                    let message = if let Some(message) = panic.downcast_ref::<String>() {
                        format!(
                            "host code panicked while being called by the runtime: {}",
                            message
                        )
                    } else if let Some(message) = panic.downcast_ref::<&'static str>() {
                        format!(
                            "host code panicked while being called by the runtime: {}",
                            message
                        )
                    } else {
                        "host code panicked while being called by the runtime".to_owned()
                    };
                    return Err(anyhow::Error::msg(message));
                }
            }
        })
    }
}

impl Function for HostFn {
    fn name(&self) -> &str {
        self.name
    }

    fn signature(&self) -> Signature {
        Signature {
            args: std::borrow::Cow::from(&[ValueType::I64]),
            return_value: Some(ValueType::I32),
        }
    }

    fn execute(
        &self,
        context: &mut dyn FunctionContext,
        args: &mut dyn Iterator<Item = Value>,
    ) -> Result<Option<Value>> {
        let fat_ptr = args
            .next()
            .and_then(|v| match v {
                Value::I64(i) => Some(i as u64),
                _ => None,
            })
            .ok_or_else(|| "Invalid argument for ptr")? as u64;
        let res_len = self.call(context, fat_ptr)?;
        Ok(Some(Value::I32(res_len as i32)))
    }
}

const HOST_FUNCTIONS: [HostFn; 2] = [
    HostFn {
        name: "host_msm_unchecked",
        func: host_msm_unchecked,
    },
    HostFn {
        name: "host_batch_hash_to_curve",
        func: batch_hash_to_curve,
    },
];

pub struct MSMHostFunctions;

impl HostFunctions for MSMHostFunctions {
    fn host_functions() -> Vec<&'static dyn Function> {
        HOST_FUNCTIONS.iter().map(|f| f as &dyn Function).collect()
    }

    fn register_static<T>(
        registry: &mut T,
    ) -> core::result::Result<(), <T as HostFunctionRegistry>::Error>
    where
        T: HostFunctionRegistry,
    {
        for func in HOST_FUNCTIONS.iter() {
            let func = *func;
            registry.register_static(
                func.name(),
                move |caller: wasmtime::Caller<T::State>, fat_ptr: u64| -> wasmtime::Result<u32> {
                    func.protected_call::<T>(caller, fat_ptr)
                },
            )?;
        }
        Ok(())
    }
}
