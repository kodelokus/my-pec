; DLLs my-pec.exe needs that Windows may not have, gathered by scripts/windows-runtime.ps1.
!define PEC_RUNTIME "${__FILEDIR__}\runtime"

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File "${PEC_RUNTIME}\onnxruntime.dll"
  File "${PEC_RUNTIME}\msvcp140.dll"
  File "${PEC_RUNTIME}\msvcp140_1.dll"
  File "${PEC_RUNTIME}\vcruntime140.dll"
  File "${PEC_RUNTIME}\vcruntime140_1.dll"
!macroend

; Before the uninstaller removes the install folder, which must be empty by then.
!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\onnxruntime.dll"
  Delete "$INSTDIR\msvcp140.dll"
  Delete "$INSTDIR\msvcp140_1.dll"
  Delete "$INSTDIR\vcruntime140.dll"
  Delete "$INSTDIR\vcruntime140_1.dll"
!macroend
