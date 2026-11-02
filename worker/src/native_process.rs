//! Native protocol modules running in a per-instance worker process.
//!
//! The worker process is a re-exec of the current executable (see [`run_child_if_requested`]).
//! It loads the module, a `no_std` shared library with its own picoalloc heap and scratch, so
//! native execution has the same memory limits as the VM backends.  Host crypto runs in the
//! worker process.  Any failure of the worker process is reported as `ModuleExecutionFailed`.

use std::{
    io::{BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
};

use codec::{Decode, Encode};
use polymesh_worker_common::{logger, *};

use crate::backend::{BackendModule, BackendModuleInstance};

/// The first argument of a worker process.
pub const NATIVE_CHILD_ARG: &str = "--polymesh-worker-native-child";

/// Bounds the scratch-sized requests/responses and the module library.
const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

#[derive(Encode, Decode)]
enum Request {
    Load {
        library: Vec<u8>,
        max_log_level: u32,
    },
    Initialize(Option<Vec<u8>>),
    SaveContext,
    Execute(Vec<u8>),
}

#[derive(Encode, Decode)]
enum Reply {
    Log {
        level: u32,
        target: Vec<u8>,
        msg: Vec<u8>,
    },
    Loaded,
    Initialized(u32),
    Context(Option<Vec<u8>>),
    Executed(WorkResponseResult),
    Failed(WorkerError),
}

fn write_msg<W: Write, T: Encode>(w: &mut W, msg: &T) -> std::io::Result<()> {
    let bytes = msg.encode();
    w.write_all(&(bytes.len() as u32).to_le_bytes())?;
    w.write_all(&bytes)?;
    w.flush()
}

fn read_msg<R: Read, T: Decode>(r: &mut R) -> std::io::Result<T> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len) as usize;
    if len > MAX_MESSAGE_SIZE {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    T::decode(&mut &buf[..]).map_err(|_| std::io::ErrorKind::InvalidData.into())
}

/// Run the worker process loop and exit, if this process was started as a worker process.
///
/// Must be called at the start of `main`, before anything writes to stdout.
pub fn run_child_if_requested() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new(NATIVE_CHILD_ARG)) {
        std::process::exit(match child::run() {
            Ok(()) => 0,
            Err(_) => 1,
        });
    }
}

/// A native module, instantiated as one worker process per instance.
pub struct NativeProcessModule {
    library: Arc<Vec<u8>>,
}

impl NativeProcessModule {
    pub fn new(library: Vec<u8>) -> Self {
        Self {
            library: Arc::new(library),
        }
    }
}

impl BackendModule for NativeProcessModule {
    fn instantiate(&self) -> Option<Box<dyn BackendModuleInstance>> {
        match NativeProcessInstance::spawn(&self.library) {
            Ok(instance) => Some(Box::new(instance)),
            Err(err) => {
                log::error!("Failed to start native worker process: {err:?}");
                None
            }
        }
    }
}

pub struct NativeProcessInstance {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl NativeProcessInstance {
    fn spawn(library: &[u8]) -> Result<Self, WorkerError> {
        let exe = std::env::current_exe().map_err(|err| {
            log::error!("Failed to get the current executable: {err}");
            WorkerError::ModuleInitializationFailed
        })?;
        let mut child = Command::new(exe)
            .arg(NATIVE_CHILD_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| {
                log::error!("Failed to spawn native worker process: {err}");
                WorkerError::ModuleInitializationFailed
            })?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(WorkerError::ModuleInitializationFailed);
        };
        let mut instance = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        };
        let load = Request::Load {
            library: library.to_vec(),
            max_log_level: logger::from_log_level_filter(log::max_level()),
        };
        match instance.call(&load)? {
            Reply::Loaded => Ok(instance),
            _ => Err(WorkerError::ModuleInitializationFailed),
        }
    }

    fn call(&mut self, request: &Request) -> Result<Reply, WorkerError> {
        if let Err(err) = write_msg(&mut self.stdin, request) {
            log::warn!("Failed to send a request to the native worker process: {err}");
            return Err(WorkerError::ModuleExecutionFailed);
        }
        loop {
            match read_msg(&mut self.stdout) {
                Ok(Reply::Log { level, target, msg }) => {
                    logger::host_logger_log(level, &target, &msg)
                }
                Ok(Reply::Failed(err)) => return Err(err),
                Ok(reply) => return Ok(reply),
                Err(err) => {
                    // The instance is discarded after this error, so reap the process for its status.
                    let _ = self.child.kill();
                    let status = match self.child.wait() {
                        Ok(status) => status.to_string(),
                        Err(err) => err.to_string(),
                    };
                    log::warn!("Native worker process failed: {err}, {status}");
                    return Err(WorkerError::ModuleExecutionFailed);
                }
            }
        }
    }
}

impl Drop for NativeProcessInstance {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl BackendModuleInstance for NativeProcessInstance {
    fn allocate_scratch_pad(&mut self, _min_size: u32) -> Result<(u32, u32), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn release_scratch_pad(&mut self, _ptr: u32, _size: u32) -> Result<(), WorkerError> {
        Ok(())
    }

    fn write_memory(&mut self, _ptr: u32, _data: &[u8]) -> Result<(), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn read_memory_into(&mut self, _ptr: u32, _buffer: &mut [u8]) -> Result<(), WorkerError> {
        Err(WorkerError::ModuleMemoryError)
    }

    fn call_initialize(&mut self, _params_len: u32, _save: u32) -> Result<u64, WorkerError> {
        Err(WorkerError::ModuleInitializationFailed)
    }

    fn call_execute(&mut self, _req_len: u32) -> Result<u64, WorkerError> {
        Err(WorkerError::ModuleExecutionFailed)
    }

    fn initialize(&mut self, load_ctx: Option<&[u8]>) -> Result<u32, WorkerError> {
        match self.call(&Request::Initialize(load_ctx.map(|ctx| ctx.to_vec())))? {
            Reply::Initialized(len) => Ok(len),
            _ => Err(WorkerError::ModuleInitializationFailed),
        }
    }

    fn save_context(&mut self) -> Result<Option<Vec<u8>>, WorkerError> {
        match self.call(&Request::SaveContext)? {
            Reply::Context(ctx) => Ok(ctx),
            _ => Err(WorkerError::ModuleSaveContextFailed),
        }
    }

    fn execute(&mut self, req: &WorkRequest) -> Result<WorkResponseResult, WorkerError> {
        match self.call(&Request::Execute(req.encode()))? {
            Reply::Executed(result) => Ok(result),
            _ => Err(WorkerError::ModuleExecutionFailed),
        }
    }
}

mod child {
    use super::*;
    use std::{
        ffi::{CStr, CString, c_void},
        fs::File,
        os::fd::FromRawFd,
        sync::{
            OnceLock,
            atomic::{AtomicU32, Ordering},
        },
    };

    /// The module memory (heap + scratch), mapped below 4 GiB for the module's u32 pointers.
    static MEMORY: OnceLock<(usize, usize)> = OnceLock::new();
    static MAX_LOG_LEVEL: AtomicU32 = AtomicU32::new(0);

    /// Lowest address tried for the module memory.
    const MEMORY_HINT: usize = 0x1000_0000;

    type MemorySizesFn = extern "C" fn() -> u64;
    type InitFn = unsafe extern "C" fn(*mut u8, *mut u8);
    type RegisterFn = unsafe extern "C" fn(*const NativeHostFunctions);
    type InitializeFn = extern "C" fn(u32, u32) -> u64;
    type ExecuteFn = extern "C" fn(u32) -> u64;

    static HOST_FUNCTIONS: NativeHostFunctions = NativeHostFunctions {
        logger_max_level: host_logger_max_level,
        logger_enabled: host_logger_enabled,
        logger_log: host_logger_log,
        msm_unchecked: host_msm_unchecked,
        batch_hash_to_curve: host_batch_hash_to_curve,
    };

    fn module_slice(fat_ptr: u64) -> Option<&'static mut [u8]> {
        let (ptr, len) = unpack_fat_pointer(fat_ptr);
        let (base, size) = *MEMORY.get()?;
        let (ptr, len) = (ptr as usize, len as usize);
        if ptr < base || ptr.checked_add(len)? > base + size {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, len) })
    }

    extern "C" fn host_logger_max_level() -> u32 {
        MAX_LOG_LEVEL.load(Ordering::Relaxed)
    }

    extern "C" fn host_logger_enabled(level: u32) -> u32 {
        (level <= MAX_LOG_LEVEL.load(Ordering::Relaxed)) as u32
    }

    extern "C" fn host_logger_log(level: u32, target_fat_ptr: u64, msg_fat_ptr: u64) {
        let (Some(target), Some(msg)) = (module_slice(target_fat_ptr), module_slice(msg_fat_ptr))
        else {
            return;
        };
        let log = Reply::Log {
            level,
            target: target.to_vec(),
            msg: msg.to_vec(),
        };
        let _ = write_msg(&mut std::io::stdout().lock(), &log);
    }

    extern "C" fn host_msm_unchecked(fat_ptr: u64) -> u32 {
        match module_slice(fat_ptr) {
            Some(buffer) => {
                let len = buffer.len() as u32;
                ark_host_msm_impl::host_msm_unchecked(buffer, len)
            }
            None => 0,
        }
    }

    extern "C" fn host_batch_hash_to_curve(fat_ptr: u64) -> u32 {
        match module_slice(fat_ptr) {
            Some(buffer) => {
                let len = buffer.len() as u32;
                bulletproofs::batch_hash_to_curve(buffer, len)
            }
            None => 0,
        }
    }

    struct Module {
        scratch: usize,
        scratch_size: usize,
        initialize: InitializeFn,
        execute: ExecuteFn,
    }

    fn dl_error() -> String {
        let err = unsafe { libc::dlerror() };
        if err.is_null() {
            "unknown error".into()
        } else {
            unsafe { CStr::from_ptr(err) }
                .to_string_lossy()
                .into_owned()
        }
    }

    unsafe fn symbol<T: Copy>(lib: *mut c_void, name: &CStr) -> Result<T, WorkerError> {
        let sym = unsafe { libc::dlsym(lib, name.as_ptr()) };
        if sym.is_null() {
            log::error!("Native module is missing symbol {name:?}");
            return Err(WorkerError::ModuleInitializationFailed);
        }
        Ok(unsafe { std::mem::transmute_copy(&sym) })
    }

    fn map_low_memory(size: usize) -> Option<usize> {
        let mut hint = MEMORY_HINT;
        while hint + size <= u32::MAX as usize {
            let ptr = unsafe {
                libc::mmap(
                    hint as *mut c_void,
                    size,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED_NOREPLACE,
                    -1,
                    0,
                )
            };
            if ptr != libc::MAP_FAILED {
                return Some(ptr as usize);
            }
            hint += MEMORY_HINT;
        }
        None
    }

    impl Module {
        fn load(library: &[u8]) -> Result<Self, WorkerError> {
            let fail = WorkerError::ModuleInitializationFailed;
            let fd = unsafe { libc::memfd_create(c"polymesh-worker-native".as_ptr(), 0) };
            if fd < 0 {
                return Err(fail);
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            file.write_all(library).map_err(|_| fail.clone())?;
            let path = CString::new(format!("/proc/self/fd/{fd}")).map_err(|_| fail.clone())?;
            let lib = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
            drop(file);
            if lib.is_null() {
                log::error!("Failed to load native module: {}", dl_error());
                return Err(fail);
            }

            unsafe {
                let memory_sizes: MemorySizesFn = symbol(lib, c"native_module_memory_sizes")?;
                let init: InitFn = symbol(lib, c"native_module_init")?;
                let register: RegisterFn = symbol(lib, c"native_module_register_host_functions")?;
                let initialize: InitializeFn = symbol(lib, c"initialize")?;
                let execute: ExecuteFn = symbol(lib, c"execute")?;

                let (heap_size, scratch_size) = unpack_fat_pointer(memory_sizes());
                let (heap_size, scratch_size) = (heap_size as usize, scratch_size as usize);
                let base = map_low_memory(heap_size + scratch_size).ok_or(fail.clone())?;
                MEMORY
                    .set((base, heap_size + scratch_size))
                    .map_err(|_| fail.clone())?;

                register(&HOST_FUNCTIONS);
                init(base as *mut u8, (base + heap_size) as *mut u8);
                Ok(Self {
                    scratch: base + heap_size,
                    scratch_size,
                    initialize,
                    execute,
                })
            }
        }

        fn write_scratch(&mut self, data: &[u8]) -> Result<u32, WorkerError> {
            if data.len() > self.scratch_size {
                return Err(WorkerError::ModuleMemoryError);
            }
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), self.scratch as *mut u8, data.len());
            }
            Ok(data.len() as u32)
        }

        fn read(fat_ptr: u64) -> Result<Vec<u8>, WorkerError> {
            module_slice(fat_ptr)
                .map(|data| data.to_vec())
                .ok_or(WorkerError::ModuleMemoryError)
        }

        fn handle(&mut self, request: Request) -> Result<Reply, WorkerError> {
            match request {
                Request::Load { .. } => Err(WorkerError::ModuleInitializationFailed),
                Request::Initialize(ctx) => {
                    let fat_ptr = match ctx {
                        Some(ctx) => {
                            let len = self.write_scratch(&ctx)?;
                            (self.initialize)(len, 0)
                        }
                        None => (self.initialize)(0, 0),
                    };
                    Ok(Reply::Initialized(unpack_fat_pointer(fat_ptr).1))
                }
                Request::SaveContext => {
                    let fat_ptr = (self.initialize)(0, 1);
                    if unpack_fat_pointer(fat_ptr).1 == 0 {
                        return Ok(Reply::Context(None));
                    }
                    Ok(Reply::Context(Some(Self::read(fat_ptr)?)))
                }
                Request::Execute(request) => {
                    let len = self.write_scratch(&request)?;
                    let result = match unpack_fat_results((self.execute)(len)) {
                        Ok((ptr, len)) => Ok(WorkResponse(Self::read(pack_fat_pointer(ptr, len))?)),
                        Err(err) => Err(err),
                    };
                    Ok(Reply::Executed(result))
                }
            }
        }
    }

    pub(super) fn run() -> std::io::Result<()> {
        // A crashing module is expected; skip the (slow) core dump.
        unsafe {
            libc::prctl(libc::PR_SET_DUMPABLE, 0);
        }
        let mut stdin = std::io::stdin().lock();
        let mut module = None;
        loop {
            // The parent closing stdin ends the worker process.
            let Ok(request) = read_msg::<_, Request>(&mut stdin) else {
                return Ok(());
            };
            let reply = match request {
                Request::Load {
                    library,
                    max_log_level,
                } if module.is_none() => {
                    MAX_LOG_LEVEL.store(max_log_level, Ordering::Relaxed);
                    Module::load(&library).map(|loaded| {
                        module = Some(loaded);
                        Reply::Loaded
                    })
                }
                request => match module.as_mut() {
                    Some(module) => module.handle(request),
                    None => Err(WorkerError::ModuleInitializationFailed),
                },
            };
            let reply = reply.unwrap_or_else(Reply::Failed);
            write_msg(&mut std::io::stdout().lock(), &reply)?;
        }
    }
}
