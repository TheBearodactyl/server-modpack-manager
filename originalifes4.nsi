Outfile "originalifes4-setup.exe"
InstallDir "$PROGRAMFILES\Originalife Season 4"
RequestExecutionLevel admin

Section
  SetOutPath $INSTDIR
  File "target\release\originalife-season4-manager.exe"
  CreateShortCut "$SMSTARTUP\Originalife Season 4.lnk" "$INSTDIR\originalife-season4-manager.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\originalife-season4-manager.exe"
  Delete "$SMSTARTUP\Originalife Season 4.lnk"
  RMDir $INSTDIR
  Delete "$INSTDIR\uninstall.exe"
SectionEnd
