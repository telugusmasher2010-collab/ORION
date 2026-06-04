$WshShell = New-Object -ComObject WScript.Shell
$DesktopPath = [Environment]::GetFolderPath("Desktop")
if (-not $DesktopPath) { $DesktopPath = "$HOME\OneDrive\Desktop" }

$Shortcut = $WshShell.CreateShortcut("$DesktopPath\ORION.lnk")
$Shortcut.TargetPath = "c:\ORION\LAUNCH_ORION.bat"
$Shortcut.WorkingDirectory = "c:\ORION"
$Shortcut.Description = "Launch ORION Dashboard"
# Points to the new high-quality icon we generated
$Shortcut.IconLocation = "c:\ORION\UI\assets\icon.png, 0"
$Shortcut.Save()

Write-Host "◆ ORION Premium Shortcut Created with Logo!" -ForegroundColor Cyan
