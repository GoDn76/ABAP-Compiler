param(
    [int]$Port = 7345,
    [string]$Url = ""
)

$target = if ($Url) { $Url } else { "http://127.0.0.1:$Port" }
Write-Host "Running ABAPC Parity Test against $target..." -ForegroundColor Cyan

# Check if target is reachable
try {
    $null = Invoke-RestMethod -Uri "$target/api/health" -TimeoutSec 3 -ErrorAction Stop
} catch {
    Write-Host "Target $target is not responding to /api/health." -ForegroundColor Yellow
    Write-Host "Make sure the Rust backend is running: 'cargo run' or 'cargo run --release'" -ForegroundColor Yellow
    exit 1
}

node "$PSScriptRoot\test-parity.mjs" --url $target
exit $LASTEXITCODE
