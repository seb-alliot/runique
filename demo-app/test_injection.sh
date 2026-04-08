# Crée le fichier de test
@"
`$BASE_URL = "https://localhost"

Write-Host "=== Test 1: Injection SQL ===" -ForegroundColor Red
`$response1 = Invoke-WebRequest -Uri "`$BASE_URL/login" -Method POST -Body @{
    username = "' OR '1'='1"
    password = "FAUX12345"
} -UseBasicParsing
Write-Host "Status: `$(`$response1.StatusCode)" -ForegroundColor Yellow
if (`$response1.Content -match "Invalid credentials") { Write-Host "Result: SECURISE (Invalid credentials)" -ForegroundColor Green }
elseif (`$response1.Content -match "Welcome") { Write-Host "Result: VULNERABLE (Welcome trouve!)" -ForegroundColor Red }
else { Write-Host "Result: A VERIFIER" -ForegroundColor Cyan }

Write-Host ""
Write-Host "=== Test 2: User inexistant ===" -ForegroundColor Blue
`$response2 = Invoke-WebRequest -Uri "`$BASE_URL/login" -Method POST -Body @{
    username = "EXISTEPAS999"
    password = "FAUX12345"
} -UseBasicParsing
Write-Host "Status: `$(`$response2.StatusCode)" -ForegroundColor Yellow
if (`$response2.Content -match "Invalid credentials") { Write-Host "Result: Normal (Invalid credentials)" -ForegroundColor Green }

Write-Host ""
Write-Host "=== Test 3: User valide, mauvais password ===" -ForegroundColor Blue
`$response3 = Invoke-WebRequest -Uri "`$BASE_URL/login" -Method POST -Body @{
    username = "Itsuki"
    password = "MAUVAIS"
} -UseBasicParsing
Write-Host "Status: `$(`$response3.StatusCode)" -ForegroundColor Yellow
if (`$response3.Content -match "Invalid credentials") { Write-Host "Result: Normal (Invalid credentials)" -ForegroundColor Green }
"@ | Out-File -FilePath "test_injection.ps1" -Encoding UTF8

