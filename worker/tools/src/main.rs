use polymesh_worker::*;
use polymesh_worker_common::{MODULE_CODE_SIZE_LIMIT, PROTOCOL_PDART};

use sp_maybe_compressed_blob::*;

fn compress_file() {
    // Parse the first command line argument as the path to the file to compress.
    let file_path = std::env::args()
        .nth(2)
        .expect("Please provide a file path to compress");

    // Read the file contents.
    let file_contents = std::fs::read(&file_path).expect("Failed to read the file");

    // Compress the file using the strongest zstd compression level.
    let compressed = compress_strongly(&file_contents, MODULE_CODE_SIZE_LIMIT)
        .expect("Failed to compress the file");

    // Write the compressed data to a new file with .zst extension.
    let compressed_file_path = format!("{}.zst", file_path);
    std::fs::write(&compressed_file_path, compressed).expect("Failed to write the compressed file");
}

fn save_protocol_context() {
    let native_backend = polymesh_worker_native::NativeBackend;
    let backends = backend::Backends::new(Some(Box::new(native_backend)));

    let mut loader = StaticModules::new();
    let protocol = Protocol {
        id: PROTOCOL_PDART,
        version: ProtocolVersion {
            major: 0,
            minor: 1,
            patch: 0,
        },
    };

    // Load the module.
    let now = std::time::Instant::now();
    let module = backends
        .load_module(protocol, &mut loader)
        .expect("No backend available for the given protocol and version");
    println!("Module loaded in: {:?}", now.elapsed());

    // Instantiate the module.
    let now = std::time::Instant::now();
    let mut instance = module.instantiate().expect("Failed to instantiate module");
    println!("Module instantiated in: {:?}", now.elapsed());

    // Initialize the module.
    {
        let now = std::time::Instant::now();
        instance
            .initialize(None)
            .expect("Failed to initialize module");
        println!("Module initialized in: {:?}", now.elapsed());
    }

    // Save the module context.
    let saved_ctx = {
        let now = std::time::Instant::now();
        let save_result = instance
            .save_context()
            .expect("Worker error during context saving");
        println!("Context saved: {}", save_result.is_some());
        println!("Time taken for saving context: {:?}", now.elapsed());

        save_result.expect("Context saving is not supported by the backend")
    };
    let hash = sp_core::blake2_256(&saved_ctx);
    println!("Saved context size: {} bytes", saved_ctx.len());
    println!("Saved context hash: 0x{}", hex::encode(hash));

    // Test loading the context back into a new instance.
    {
        let now = std::time::Instant::now();
        instance
            .initialize(Some(saved_ctx.as_ref()))
            .expect("Failed to initialize with saved context");
        println!("Time taken for loading context: {:?}", now.elapsed());
    }

    // Save the context to a file.
    let saved_ctx_file_path = "polymesh-worker-protocol-dart-v0.context.bin";
    std::fs::write(&saved_ctx_file_path, saved_ctx)
        .expect("Failed to write the saved context to a file");
}

pub fn main() {
    env_logger::init();

    // Get first cli argument as the command to run (compress, or save protocol context).
    let command = std::env::args()
        .nth(1)
        .expect("Please provide a command to run (compress or save-context)");
    if command == "compress" {
        compress_file();
    } else if command == "save-context" {
        save_protocol_context();
    } else {
        eprintln!("ERROR: Unknown command: {}", command);
    }
}
