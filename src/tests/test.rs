
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn wat_wasm_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("runtime-fuzzer")?;

    cmd.arg("--wat").arg("test/file/doesnt/exist");
    cmd.arg("--wasm").arg("test/file/doesnt/exist");
    cmd.arg("--contract").arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(
        predicate::str::contains("Please specify exactly one of --wat, --contract, or --wasm")
    );

    Ok(())
}
