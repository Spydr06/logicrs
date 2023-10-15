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
$installer = "logicrs-windows-x86_64-installer.exe"

function Create-Shortcut {
    param (
        [string]$SourceExe,
        [string]$Arguments,
        [string]$IconFile,
        [string]$DestinationPath
    )

    Write-Output "Info: Creating shortcut $DestinationPath to $SourceExe."
    $WshShell = New-Object -ComObject ("WScript.Shell")
    $Shortcut = $WshShell.CreateShortcut($DestinationPath)
    $Shortcut.TargetPath = $SourceExe
    $Shortcut.Arguments = $Arguments
    $Shortcut.IconLocation = $IconFile
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
    Test-Remove -Path $installer

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
            "share\logicrs.ico"                   `
            "$(Resolve-Path -Path ".")\logicrs.lnk"

        # New-Item -ItemType SymbolicLink -Path ".\$executable" -Target ".\bin\$executable"

        rcedit ".\bin\$executable" --set-icon ".\share\logicrs.ico"
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

    Write-Output "Info: Generating Installer using NSIS."

    $snippets_dir = "snippets"
    makensis.exe "$snippets_dir\installer.nsi"

    Move-Item -Path "$snippets_dir\$installer" -Destination $installer

    Write-Output "Info: Generated $installer."
Pop-Location # script directory
