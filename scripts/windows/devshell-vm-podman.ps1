#Requires -Version 5.1
<#
.SYNOPSIS
  Build the Linux devshell-vm binary for Windows beta (Podman Machine stdio).

.DESCRIPTION
  Default Windows beta flow uses: podman machine ssh -> devshell-vm --serve-stdio (no host TCP).
  This script runs cargo build with target x86_64-unknown-linux-gnu and prints
  recommended env vars. It does NOT start a podman run -p 9847 sidecar by default.

  For optional TCP-in-container debugging, see containers/devshell-vm/README.md.

.PARAMETER RepoRoot
  Path to xtask_todo repository root (default: two levels above this script).
#>
param(
    [string] $RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
}

$podman = Get-Command podman -ErrorAction SilentlyContinue
if (-not $podman) {
    Write-Warning "podman not found in PATH. Install Podman for Windows: https://podman.io/"
}

# Same as cargo-devshell: Podman Go/SSH uses %USERPROFILE%\.ssh\known_hosts (not only $HOME).
if (-not $env:DEVSHELL_VM_DISABLE_PODMAN_SSH_HOME) {
    $realProfile = $env:USERPROFILE
    $sshHome = Join-Path $env:TEMP "cargo-devshell-ssh-home"
    $null = New-Item -ItemType Directory -Force -Path (Join-Path $sshHome ".ssh")
    $kh = Join-Path $sshHome ".ssh\known_hosts"
    if (-not (Test-Path -LiteralPath $kh)) {
        New-Item -ItemType File -Force -Path $kh | Out-Null
    }
    $realPodman = Join-Path $realProfile ".local\share\containers\podman"
    $linkPodman = Join-Path $sshHome ".local\share\containers\podman"
    if ((Test-Path -LiteralPath $realPodman -PathType Container) -and -not (Test-Path -LiteralPath $linkPodman)) {
        $null = New-Item -ItemType Directory -Force -Path (Split-Path $linkPodman -Parent)
        try {
            New-Item -ItemType SymbolicLink -Path $linkPodman -Target $realPodman -ErrorAction Stop | Out-Null
        }
        catch {
            Write-Warning "Could not symlink Podman machine into isolated USERPROFILE (enable Developer Mode or run elevated): $_"
        }
    }
    $env:USERPROFILE = $sshHome
    $env:HOME = $sshHome
    if ($sshHome.Length -ge 2 -and $sshHome[1] -eq ':') {
        $env:HOMEDRIVE = $sshHome.Substring(0, 2)
        $env:HOMEPATH = $sshHome.Substring(2)
    }
}

Push-Location $RepoRoot
try {
    & rustup target add x86_64-unknown-linux-gnu
    & cargo build -p devshell-vm --release --target x86_64-unknown-linux-gnu
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

$bin = Join-Path $RepoRoot "target\x86_64-unknown-linux-gnu\release\devshell-vm"
Write-Host "Linux devshell-vm built: $bin" -ForegroundColor Green
Write-Host "Recommended for cargo-devshell (default stdio + Podman Machine):" -ForegroundColor Yellow
Write-Host '  $env:DEVSHELL_VM_BACKEND = "beta"'
Write-Host '  (omit DEVSHELL_VM_SOCKET or set to "stdio")'
Write-Host "  `$env:DEVSHELL_VM_LINUX_BINARY = `"$bin`""
Write-Host '  $env:DEVSHELL_VM_WORKSPACE_PARENT = "<your workspace root>"'
Write-Host "See docs/devshell-vm-windows.md"
exit 0
