# The single verification script (onboarding R13). Stops at the first failing gate.
# -E2E also builds the app with the `e2e` feature and runs the end-to-end specs against the replay server.
param([switch]$E2E)

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

function Step([string]$name, [scriptblock]$run) {
    Write-Host "==> $name" -ForegroundColor Cyan
    $global:LASTEXITCODE = 0
    & $run
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $name" -ForegroundColor Red
        exit 1
    }
}

Step 'cargo fmt' { cargo fmt --all --check }
Step 'cargo clippy' { cargo clippy --workspace --all-targets --locked -- -D warnings }
Step 'pec-core and pec-ext have no Tauri dependency (Principle VI)' {
    $found = 'pec-core', 'pec-ext' | ForEach-Object { cargo tree -p $_ --edges all --prefix none --locked } |
        Select-String -Pattern '^tauri[ -]'
    if ($found) {
        Write-Host $found
        $global:LASTEXITCODE = 1
    }
}
Step 'cargo test' { cargo test --workspace --locked }
Step 'generated TypeScript types match the committed ones' {
    git diff --exit-code -- src/bindings
    if ($LASTEXITCODE -eq 0) {
        $untracked = git ls-files --others --exclude-standard -- src/bindings
        if ($untracked) {
            Write-Host "Not committed: $untracked"
            $global:LASTEXITCODE = 1
        }
    }
}
Step 'biome' { npm run --silent check }
Step 'typescript' { npm run --silent typecheck }
Step 'vitest' { npm run --silent test }
# avatar R3: the Live2D renderer is type-checked only where Live2D's SDK is present.
if (Test-Path vendor/cubism/Framework/src) {
    Step 'typescript (Live2D renderer)' { npx tsc --noEmit -p tsconfig.live2d.json }
}

if ($E2E) {
    Step 'build the app for end-to-end runs' { npx tauri build --debug --no-bundle --features e2e }
    Step 'end-to-end specs' { npm run --silent e2e }
}

Write-Host 'All gates passed.' -ForegroundColor Green
