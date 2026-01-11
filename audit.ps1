# audit.ps1
# Security audit with known unfixable vulnerabilities ignored
# Run: .\audit.ps1

Write-Host "Running security audit..." -ForegroundColor Cyan

cargo audit `
  --ignore RUSTSEC-2023-0071 `
  --ignore RUSTSEC-2025-0052 `
  --ignore RUSTSEC-2024-0384

if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Security audit passed!" -ForegroundColor Green
} else {
    Write-Host "[FAILED] Security audit failed!" -ForegroundColor Red
    exit 1
}