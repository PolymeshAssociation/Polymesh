use anyhow::{anyhow, Result};
use integration::PolymeshTester;
use sp_weights::Weight;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(anyhow!("Usage: {} <command> [args...]", args[0]));
    }

    match args[1].as_str() {
        "upgrade_chain" => {
            if args.len() != 3 {
                return Err(anyhow!("Usage: {} upgrade_chain <wasm_file>", args[0]));
            }
            upgrade_chain(&args[2]).await
        }
        _ => Err(anyhow!("Unknown command: {}", args[1])),
    }
}

async fn upgrade_chain(wasm_path: &str) -> Result<()> {
    let tester = PolymeshTester::new().await?;
    let mut sudo = tester.sudo.clone().expect("No Sudo user");

    // Read the WASM file
    let code = fs::read(wasm_path)?;

    // Set code call
    let set_code = tester.api.call().system().set_code(code.clone())?;

    // Create and submit the upgrade transaction
    let mut res = tester
        .api
        .call()
        .sudo()
        .sudo_unchecked_weight(set_code.into(), Weight::from_parts(1_000_000_000, 0))?
        .submit_and_watch(&mut sudo)
        .await?;

    // Wait for finalization
    res.wait_finalized().await?;
    println!("Chain upgrade completed successfully");

    Ok(())
}
