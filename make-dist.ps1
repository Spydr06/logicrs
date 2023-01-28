# PowerShell script for creating a distributable version of LogicRs
# LICENSE: MIT
# Author: Spydr06
# Repository: https://github.com/Spydr06/logicrs
#

# configuration
$dist_dir = "windows-dist"
$license = "LICENSE"
$readme = "README.md"
$executable = "logicrs.exe"
$shortcut = "logicrs.lnk"
$target = ".\target\release\$executable"
$zip_name = "logicrs-win64.zip"
$strip_debug_script = "strip-debug-symbols.ps1"

function Create-Shortcut {
    param (
        [string]$SourceExe,
        [string]$Arguments,
        [string]$DestinationPath
    )

    Write-Output "Info: Creating shortcut $DestinationPath to $SourceExe."
    $WshShell = New-Object -ComObject ("WScript.Shell")
    $Shortcut = $WshShell.CreateShortcut($DestinationPath)
    $Shortcut.TargetPath = $SourceExe
    $Shortcut.Arguments = $Arguments
    $Shortcut.Save()
}

function Test-Remove {
    param (
        [string]$Path
    )

    if (Test-Path $Path) {
        Write-Warning "Warn: Removing old file $Path."
        Remove-Item -Path $Path
    }
}

# go to script dir
$scriptpath = $MyInvocation.MyCommand.Path
$dir = Split-Path $scriptpath
Push-Location $dir
    # delete old files
    Test-Remove -Path $zip_name
    Test-Remove -Path "$dist_dir\$shortcut"
    Test-Remove -Path "$dist_dir\bin\$executable"

    # compile for release
    cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"

    if (!(Test-Path $target)) {
        Write-Error "Error: $target not found."
        Pop-Location
        exit 1
    }

    if (!(Test-Path $dist_dir)) {
        Write-Error "Error: $dist_dir not found."
        Pop-Location
        exit 1
    }

    # copy files over to the windows-dist folder
    Copy-Item -Path $target -Destination "$dist_dir\bin\$executable"
    Copy-Item -Path $license -Destination "$dist_dir\$license"
    Copy-Item -Path $readme -Destination "$dist_dir\$readme"

    # enter the windows-dist folder
    Push-Location $dist_dir
        Push-Location "bin"
            # strip all debug information
            & ".\$strip_debug_script"
        Pop-Location # bin directory

        # create shortcut to top level
        Create-Shortcut                             `
            "%windir%\explorer.exe"                 `
            ".\bin\$executable"                     `
            "$(Resolve-Path -Path ".")\logicrs.lnk"
    Pop-Location # dist directory

    # create the distributable zip archive
    Compress-Archive -Path "$dist_dir\*" -Update -DestinationPath $zip_name

    if (!(Test-Path $zip_name)) {
        Write-Error "Error: $zip_name not created."
        Pop-Location
        exit 1
    }
    else {
        Write-Output "Info: Created distributable package $zip_name."
    }
Pop-Location # script directory
