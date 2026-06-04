' ORION — Create Desktop Shortcut + Start Menu Entry
' Supports "Pin to Taskbar" and "Pin to Start"

Set oWS = WScript.CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")

' Desktop shortcut
desktopPath = oWS.ExpandEnvironmentStrings("%USERPROFILE%") & "\OneDrive\Desktop"
If Not fso.FolderExists(desktopPath) Then
    desktopPath = oWS.ExpandEnvironmentStrings("%USERPROFILE%") & "\Desktop"
End If

' Start Menu shortcut
startMenuPath = oWS.ExpandEnvironmentStrings("%APPDATA%") & "\Microsoft\Windows\Start Menu\Programs"
If Not fso.FolderExists(startMenuPath) Then
    startMenuPath = oWS.ExpandEnvironmentStrings("%PROGRAMDATA%") & "\Microsoft\Windows\Start Menu\Programs"
End If

' Create Desktop shortcut
sLinkFile = desktopPath & "\ORION.lnk"
Set oLink = oWS.CreateShortcut(sLinkFile)
oLink.TargetPath = "C:\ORION\node_modules\electron\dist\electron.exe"
oLink.Arguments = "."
oLink.WorkingDirectory = "C:\ORION"
oLink.IconLocation = "C:\ORION\UI\assets\icon.ico,0"
oLink.Description = "ORION - Personal AI Assistant"
oLink.Save

' Create Start Menu shortcut (this enables "Pin to Taskbar")
sLinkFile2 = startMenuPath & "\ORION.lnk"
Set oLink2 = oWS.CreateShortcut(sLinkFile2)
oLink2.TargetPath = "C:\ORION\node_modules\electron\dist\electron.exe"
oLink2.Arguments = "."
oLink2.WorkingDirectory = "C:\ORION"
oLink2.IconLocation = "C:\ORION\UI\assets\icon.ico,0"
oLink2.Description = "ORION - Personal AI Assistant"
oLink2.Save

WScript.Echo "ORION shortcuts created!" & vbCrLf & vbCrLf & _
    "Desktop: " & sLinkFile & vbCrLf & _
    "Start Menu: " & sLinkFile2 & vbCrLf & vbCrLf & _
    "Right-click the taskbar icon while ORION is running to Pin to Taskbar." & vbCrLf & _
    "Right-click ORION in Start Menu to Pin to Start."
