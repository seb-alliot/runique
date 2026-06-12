<#
  Smoke-test multi-moteur de makemigrations (version PowerShell, machines lentes a compiler).

  Pour chaque moteur demande (sqlite, postgres, mariadb) :
    1. clean slate : suppression des migrations + snapshots + lib.rs de demo-app
    2. regeneration via `runique makemigrations` AVEC le DbKind du moteur
       (CREATE TYPE/triggers pour PG, ENUM inline pour MariaDB, TEXT pour SQLite)
    3. execution reelle : `cargo run -p migration -- fresh` puis `reset`

  Les migrations de demo-app sont sauvegardees au debut et RESTAUREES a la fin.

  Optimisation laptop lent :
    * la lib `runique` (le gros) est compilee UNE SEULE FOIS ; le binaire CLI est
      ensuite appele directement (pas de `cargo run`).
    * seule la petite crate `migration` recompile entre les moteurs.

  Usage :
    powershell -ExecutionPolicy Bypass -File scripts\smoke_migrations.ps1 sqlite
    powershell -ExecutionPolicy Bypass -File scripts\smoke_migrations.ps1
    powershell -ExecutionPolicy Bypass -File scripts\smoke_migrations.ps1 postgres mariadb

  Override URLs : $env:DATABASE_URL_PG / $env:DATABASE_URL_MARIADB
#>

$ErrorActionPreference = "Continue"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$MigSrc     = Join-Path $Root "demo-app\migration\src"
$RuniqueBin = Join-Path $Root "target\debug\runique.exe"

$PgUrl    = if ($env:DATABASE_URL_PG)      { $env:DATABASE_URL_PG }      else { "postgres://runique:runique_test@localhost:5433/runique_test" }
$MariaUrl = if ($env:DATABASE_URL_MARIADB) { $env:DATABASE_URL_MARIADB } else { "mysql://runique:runique_test@localhost:3307/runique_test" }

# Forme canonique Runique : `sqlite://<nom_relatif>?mode=rwc` (cf. db/config.rs).
# Un chemin ABSOLU casse le parsing d'URL sur Windows (le `C:` est pris pour un host).
# Le nom relatif est cree dans le cwd au moment du `cargo run` (= $Root).
$SqliteName = "runique_smoke_{0}.db" -f ([guid]::NewGuid().ToString("N"))
$SqliteFile = Join-Path (Join-Path $Root "demo-app") $SqliteName  # cree dans le cwd (demo-app)
# Concatenation explicite : "$SqliteName?mode" est mal tokenise par PowerShell (le `?` colle
# au nom de variable) et avale le nom de fichier -> URL cassee `sqlite://=rwc`.
$SqliteUrl  = 'sqlite://' + $SqliteName + '?mode=rwc'

# Moteurs demandes (args), sinon les 3.
$Engines = $args
if ($Engines.Count -eq 0) { $Engines = @("sqlite", "postgres", "mariadb") }
function Want($name) { return ($Engines -contains $name) }

# Sauvegarde de l'environnement DB pour le restaurer ensuite.
$OrigDbUrl     = $env:DB_URL
$OrigDbUrl2    = $env:DATABASE_URL
$OrigDbEngine  = $env:DB_ENGINE

# Sauvegarde des migrations committees.
$Backup = Join-Path $env:TEMP ("runique_mig_backup_{0}" -f ([guid]::NewGuid()))
Copy-Item -Recurse -Path $MigSrc -Destination $Backup

# Le binaire runique fait `dotenv_override()` : le .env de demo-app ECRASE les variables
# d'env. On le pilote donc par moteur (sauvegarde + restauration).
$EnvFile = Join-Path $Root "demo-app\.env"
$EnvBackup = $null
if (Test-Path $EnvFile) {
    $EnvBackup = "$EnvFile.smoke-bak"
    Copy-Item -Path $EnvFile -Destination $EnvBackup -Force
}

function Clear-Migrations {
    # Migration files are `m<digits>_*.rs` — must NOT match `main.rs` (the crate's bin).
    Get-ChildItem -Path $MigSrc -Filter "*.rs" -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '^m\d' } | Remove-Item -Force
    $snap = Join-Path $MigSrc "snapshots"
    if (Test-Path $snap) { Remove-Item -Recurse -Force $snap }
    $lib = Join-Path $MigSrc "lib.rs"
    if (Test-Path $lib) { Remove-Item -Force $lib }
}

function Invoke-Engine {
    param([string]$Name, [string]$Url, [string]$Engine)
    Write-Host ""
    Write-Host "================== $Name =================="

    Clear-Migrations

    # .env de demo-app force pour CE moteur (dotenv_override charge ce fichier).
    Set-Content -Path $EnvFile -Encoding ascii -Value @(
        "DB_ENGINE=$Engine",
        "DB_URL=$Url",
        "DATABASE_URL=$Url"
    )

    # Tout depuis demo-app : makemigrations ET migrate, pour que le chemin SQLite
    # relatif se resolve dans demo-app (comme un usage reel du projet).
    $env:DB_URL = $Url; $env:DATABASE_URL = $Url; $env:DB_ENGINE = $Engine
    Push-Location (Join-Path $Root "demo-app")
    try {
        Write-Host "--- makemigrations ($Name) ---"
        & $RuniqueBin makemigrations --entities src/entities --migrations migration/src | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Host "ECHEC: makemigrations ($Name)"; return $false }

        Write-Host "--- migrate fresh ($Name) ---"
        cargo run -q -p migration -- fresh | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Host "ECHEC: fresh ($Name)"; return $false }

        Write-Host "--- migrate reset ($Name) ---"
        cargo run -q -p migration -- reset | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Host "ECHEC: reset ($Name)"; return $false }
    }
    finally {
        Pop-Location
    }

    Write-Host "OK: $Name"
    return $true
}

$Result = @{}

try {
    # Build unique du CLI runique (compile la lib lourde une seule fois).
    Write-Host "--- build CLI runique (une fois) ---"
    cargo build -q -p runique --bin runique
    if ($LASTEXITCODE -ne 0) { Write-Host "ECHEC: build du CLI runique"; exit 1 }

    # Docker uniquement si PG/MariaDB sont demandes.
    if ((Want "postgres") -or (Want "mariadb")) {
        $hasDocker = Get-Command docker -ErrorAction SilentlyContinue
        if ($hasDocker -and (Test-Path (Join-Path $Root "docker-compose.yml"))) {
            $svc = @()
            if (Want "postgres") { $svc += "postgres" }
            if (Want "mariadb")  { $svc += "mariadb" }
            Write-Host "--- docker compose up -d --wait $($svc -join ' ') ---"
            docker compose up -d --wait @svc
            if ($LASTEXITCODE -ne 0) { Write-Host "ATTENTION: docker compose a echoue - moteur(s) concerne(s) echoueront s'ils ne repondent pas" }
        }
    }

    if (Want "sqlite")   { $Result["SQLite"]   = (Invoke-Engine "SQLite"   $SqliteUrl "sqlite") }
    if (Want "postgres") { $Result["Postgres"] = (Invoke-Engine "Postgres" $PgUrl     "postgres") }
    if (Want "mariadb")  { $Result["MariaDB"]  = (Invoke-Engine "MariaDB"  $MariaUrl  "mariadb") }
}
finally {
    Remove-Item -Recurse -Force $MigSrc
    Copy-Item -Recurse -Path $Backup -Destination $MigSrc
    Remove-Item -Recurse -Force $Backup
    foreach ($f in @($SqliteFile, "$SqliteFile-wal", "$SqliteFile-shm")) {
        if (Test-Path $f) { Remove-Item -Force $f }
    }
    # Restaure le .env de demo-app.
    if ($EnvBackup -and (Test-Path $EnvBackup)) {
        Move-Item -Path $EnvBackup -Destination $EnvFile -Force
    } elseif (Test-Path $EnvFile) {
        Remove-Item -Force $EnvFile
    }
    $env:DB_URL = $OrigDbUrl; $env:DATABASE_URL = $OrigDbUrl2; $env:DB_ENGINE = $OrigDbEngine
    Write-Host "--- migrations demo-app restaurees ---"
}

Write-Host ""
Write-Host "==================== RESUME ===================="
$fail = 0
foreach ($engine in @("SQLite", "Postgres", "MariaDB")) {
    if (-not $Result.ContainsKey($engine)) { continue }
    $status = if ($Result[$engine]) { "OK" } else { "ECHEC" }
    "{0,-10} : {1}" -f $engine, $status | Write-Host
    if (-not $Result[$engine]) { $fail = 1 }
}
Write-Host "================================================"

exit $fail
