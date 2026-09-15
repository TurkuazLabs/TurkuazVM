@echo off
REM # 📄 Dosya Yolu: /turkuazvm/TurkuazVM-Recovery-Verify.cmd
REM # 📌 Amac: Windows kullanicisi icin FULL recovery dogrulamasini tek tikla baslatir
REM # 📌 Modul - CMD
REM # Version: 0.28.0
REM # Aciklama: PowerShell recovery verify scriptini proje kokunden calistirir
REM # Bagimli Oldugu Katman: Tool | View
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\verify_recovered_full.ps1"
set EXITCODE=%ERRORLEVEL%
echo.
if not "%EXITCODE%"=="0" (
  echo TURKUAZVM_FULL_RECOVERY_VERIFY=BLOCKED exit=%EXITCODE%
) else (
  echo TURKUAZVM_FULL_RECOVERY_VERIFY=PASS
)
pause
exit /b %EXITCODE%
