#Requires -Version 5.1
<#
.SYNOPSIS
  Build and run the devshell-vm (beta) sidecar in Podman on Windows.

.DESCRIPTION
  Mounts a host workspace directory into the container at /workspace and publishes TCP 9847.
  Set these before starting cargo-devshell (same shell or user environment):
    $env:DEVSHELL_VM_BACKEND = "beta"
    $env:DEVSHELL_VM_SOCKET = "tcp:127.0.0.1:9847"
    $env:DEVSHELL_VM_BETA_SESSION_STAGING = "/workspace"
    $env:DEVSHELL_VM_WORKSPACE_PARENT = "<HostWorkspace>"   # same folder as -HostWorkspace

.PARAMETER HostWorkspace
  Absolute Windows path to the workspace root (must match DEVSHELL_VM_WORKSPACE_PARENT for cargo-devshell).

.PARAMETER Port
  Host port to map to the sidecar (default 9847).

.PARAMETER ImageTag
  Local image tag after build (default devshell-vm:local).

.PARAMETER RepoRoot
  Path to xtask_todo repository root (default: two levels above this script).
#>
param(
    [Parameter(Mandatory = $true)]
    [string] $HostWorkspace,

    [int] $Port = 9847,
    [string] $ImageTag = "devshell-vm:local",
    [string] $RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
}

$ws = [System.IO.Path]::GetFullPath($HostWorkspace)
if (-not (Test-Path -LiteralPath $ws -PathType Container)) {
    Write-Error "HostWorkspace is not a directory: $ws"
}

$podman = Get-Command podman -ErrorAction SilentlyContinue
if (-not $podman) {
    Write-Error "podman not found in PATH. Install Podman for Windows: https://podman.io/"
}

# Same as cargo-devshell: Podman Go/SSH uses %USERPROFILE%\.ssh\known_hosts (not only $HOME).
# Isolate USERPROFILE to %TEMP%\cargo-devshell-ssh-home with an empty known_hosts when the real file is locked/invalid.
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
    & podman build -f "containers/devshell-vm/Containerfile" -t $ImageTag .
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

# Podman on Windows: convert path for -v (often accepts C:\... style)
Write-Host "Starting sidecar: ${ImageTag} -> tcp/127.0.0.1:$Port (container /workspace <- host $ws)" -ForegroundColor Cyan
Write-Host "Set for cargo-devshell:" -ForegroundColor Yellow
Write-Host '  $env:DEVSHELL_VM_BACKEND = "beta"'
Write-Host "  `$env:DEVSHELL_VM_SOCKET = `"tcp:127.0.0.1:$Port`""
Write-Host '  $env:DEVSHELL_VM_BETA_SESSION_STAGING = "/workspace"'
Write-Host "  `$env:DEVSHELL_VM_WORKSPACE_PARENT = `"$ws`""

& podman run --rm -p "${Port}:9847" -v "${ws}:/workspace" $ImageTag
exit $LASTEXITCODE
