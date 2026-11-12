# Run the Rust backend verification against the counterpart smoke test suite
node "$PSScriptRoot\check.mjs"
exit $LASTEXITCODE
