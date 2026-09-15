@rem # 📄 Dosya Yolu: /turkuazvm/TurkuazVM-Start.cmd
@rem # 📌 Amac: TurkuazVM Windows test launcher PowerShell aracini tek tikla baslatir
@rem # 📌 Modul - CMD
@rem # Version: 0.37.3
@rem # Aciklama: Portable QEMU Turkuaz NAT kullanan Windows PowerShell launcher scriptini calistirir
@rem # Bagimli Oldugu Katman: Tool | View
@echo off
setlocal
pushd "%~dp0"
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start_turkuazvm.ps1"
set "TURKUAZVM_EXIT=%ERRORLEVEL%"
popd
exit /b %TURKUAZVM_EXIT%
