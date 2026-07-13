#!/usr/bin/env pwsh

$ErrorActionPreference = 'Stop'

$iscc = Get-Command -Name ISCC -CommandType Application -ErrorAction SilentlyContinue
if (-not $iscc) {
    throw 'ISCC was not found. Install Inno Setup 6.5.0 or newer.'
}

$check_root = Join-Path ([System.IO.Path]::GetTempPath()) "warp-installer-check-$([guid]::NewGuid())"
$target_dir = Join-Path $check_root 'target'
$resources_dir = Join-Path $target_dir 'resources'
$output_dir = Join-Path $check_root 'output'
$output_name = 'WarpInstallerCheck'

try {
    New-Item -ItemType Directory -Path $resources_dir, $output_dir | Out-Null
    Set-Content -LiteralPath (Join-Path $target_dir 'warp-oss.exe') -Value 'MZ' -NoNewline -Encoding ascii
    Set-Content -LiteralPath (Join-Path $resources_dir 'placeholder.txt') -Value 'installer check' -Encoding ascii

    $iscc_args = @(
        (Join-Path $PSScriptRoot 'windows-installer.iss'),
        '/Qp',
        "/O$output_dir",
        '/DReleaseChannel=oss',
        '/DMyAppExeName=warp-oss.exe',
        "/DTargetProfileDir=$target_dir",
        '/DMyAppName=WarpOss',
        '/DMyAppVersion=0.0.0-ci',
        '/DArch=x64',
        "/DOutputName=$output_name"
    )

    & $iscc.Source @iscc_args
    if ($LASTEXITCODE -ne 0) {
        throw "ISCC failed with exit code $LASTEXITCODE"
    }

    $installer_path = Join-Path $output_dir "$output_name.exe"
    if (-not (Test-Path -LiteralPath $installer_path -PathType Leaf)) {
        throw "ISCC did not create the expected installer at $installer_path"
    }

    Write-Output "Validated Windows installer at $installer_path"
} finally {
    if (Test-Path -LiteralPath $check_root) {
        Remove-Item -LiteralPath $check_root -Recurse -Force
    }
}
