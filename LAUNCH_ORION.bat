@echo off
cd /d C:\ORION

echo ◆ ORION System Starting...

REM Always recreate desktop shortcut (ensures icon is fresh)
echo [1/3] Creating desktop shortcut...
cscript //nologo "%~dp0create-shortcut.vbs"

REM Check if Electron exists
if not exist "C:\ORION\node_modules\electron\dist\electron.exe" (
    echo [ERROR] Electron not found. Run: npm install
    pause
    exit
)

echo [2/3] Launching ORION...
start "" "C:\ORION\node_modules\electron\dist\electron.exe" .
echo [3/3] Done. ORION is running.
exit
