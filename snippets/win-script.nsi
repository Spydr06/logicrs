Outfile "logicrs-windows-x86_64.exe"
Section
    StrCpy $INSTDIR $EXEDIR
    StrCpy $INSTDIR "$INSTDIR\logicrs"
    SetOutPath $INSTDIR
    File /r "F:\exe\logicrs-windows-x86_64\*.*"
SectionEnd
