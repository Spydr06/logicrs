!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON "..\windows-dist\share\logicrs.ico"

Var StartMenuFolder

Name "LogicRs"
Outfile "logicrs-windows-x86_64-installer.exe"
Unicode True
InstallDir "C:\Program Files\logicrs"

!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY

!define MUI_STARTMENUPAGE_REGISTRY_ROOT "HKCU" 
!define MUI_STARTMENUPAGE_REGISTRY_KEY "Software\Modern UI Test" 
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "Start Menu Folder"
!insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder

!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"    
    SetOutPath $INSTDIR
    File /r "..\windows-dist\*.*"

    WriteRegStr HKCU "Software\LogicRs" "" "$INSTDIR"
    WriteUninstaller "$INSTDIR\uninstall.exe"


    !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
        CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
        CreateShortcut "$SMPROGRAMS\$StartMenuFolder\LogicRs.lnk" "$INSTDIR\bin\logicrs.exe"
        CreateShortcut "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk" "$INSTDIR\uninstall.exe"
    !insertmacro MUI_STARTMENU_WRITE_END
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR"

    !insertmacro MUI_STARTMENU_GETFOLDER Application $StartMenuFolder
    Delete "$SMPROGRAMS\$StartMenuFolder\LogicRs.lnk"
    Delete "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk"
    RmDir "$SMPROGRAMS\$StartMenuFolder"

    DeleteRegKey /ifempty HKCU "Software\LogicRs"
SectionEnd
