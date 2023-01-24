# powershell script to strip all debug information
function Strip-Binaries {
	param (
		$Path
	)

	Push-Location $Path
		Get-ChildItem * -Include *.dll,*.exe | 
		Foreach-Object {
			Write-Output "Info: Stripping debug information from: $_"
			strip.exe --strip-all -g -S -d --strip-debug $_
		}
	Pop-Location
}

Strip-Binaries -Path "."
Strip-Binaries -Path "..\lib\gdk-pixbuf-2.0\2.10.0\loaders"
