use sc_executor::{
	WasmExecutor, WasmExecutionMethod,
  WasmtimeInstantiationStrategy,
};
use sp_state_machine::BasicExternalities;
use sp_core::{
	traits::{CallContext, CodeExecutor, WrappedRuntimeCode, RuntimeCode},
};
use std::borrow::Cow;
use std::sync::Arc;

type HostFunctions = sp_io::SubstrateHostFunctions;

fn test_runtime() -> &'static [u8] {
	include_bytes!("polymesh_testnet_5004001.wasm")
}

fn wrap_runtime<'a>(code: &'a [u8]) -> WrappedRuntimeCode<'a> {
	WrappedRuntimeCode(Cow::from(code))
}

fn read_runtime<'a>(code: &'a WrappedRuntimeCode<'a>, hash: u32) -> RuntimeCode<'a> {
	RuntimeCode {
		code_fetcher: code,
		heap_pages: Some(2048),
		hash: hash.to_le_bytes().to_vec(),
	}
}

fn init_executor(method: WasmExecutionMethod) -> Arc<WasmExecutor<HostFunctions>> {
	Arc::new(WasmExecutor::new(method, Some(2048), 8, None, 2))
}

fn call_runtime_version<'a>(
  executor: &Arc<WasmExecutor<HostFunctions>>,
	runtimes: &'a [(WrappedRuntimeCode<'a>, u32)],
) {
	let mut ext = BasicExternalities::new_empty();
	for (runtime, hash) in runtimes {
		let code = read_runtime(runtime, *hash);
		let (res, _) = executor
			.call(&mut ext, &code, "Core_version", &[], false, CallContext::Onchain);
		assert!(res.is_ok());
	}
}

fn main() {
	env_logger::init();

	let test = test_runtime();
	let main_runtimes = [
		(wrap_runtime(test), 1),
		(wrap_runtime(test), 2),
		(wrap_runtime(test), 3),
  ];

	let runtimes = [
		(wrap_runtime(test), 1),
		(wrap_runtime(test), 2),
		(wrap_runtime(test), 3),
	];

	let main_executor = init_executor(WasmExecutionMethod::Compiled {
		instantiation_strategy: WasmtimeInstantiationStrategy::PoolingCopyOnWrite,
	});
  {
  	let executor = init_executor(WasmExecutionMethod::Compiled {
  		instantiation_strategy: WasmtimeInstantiationStrategy::PoolingCopyOnWrite,
  	});
  
    let threads: Vec<_> = runtimes.into_iter().enumerate()
      .map(|(id, (runtime, hash))| {
        let executor = executor.clone();
        std::thread::spawn(move || {
          let runtimes = [(runtime, hash)];
          println!("started thread[{id}]");
          for idx in 0..2 {
            println!("thread[{id}] Run[{idx}]");
  	        call_runtime_version(&executor, &runtimes);
          }
          println!("stopped thread[{id}]");
        })
      })
      .collect();

    println!("Wait for threads to stop.");
    for thread in threads {
      thread.join().unwrap();
    }
  }
  for idx in 0..2 {
    println!("main Run[{idx}]");
	  call_runtime_version(&main_executor, &main_runtimes);
  }
  println!("Finished");
}
