; ==============================================================================
; MySession — Script Hook NSIS (Windows 11 Setup)
; Autore: Riccardo Bettini
; ==============================================================================

!macro NSIS_HOOK_POSTINSTALL
  ; Crea le cartelle dedicate per i modelli AI e i binari in %APPDATA%
  CreateDirectory "$APPDATA\com.mysession.app"
  CreateDirectory "$APPDATA\com.mysession.app\models"
  CreateDirectory "$APPDATA\com.mysession.app\bin"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Operazioni pre-disinstallazione
!macroend
