$ErrorActionPreference = "SilentlyContinue"
Stop-Service sshd -Force
Set-Service sshd -StartupType Disabled
Remove-NetFirewallRule -Name "TestSSH-LAN"
Remove-Item "$env:ProgramData\ssh\administrators_authorized_keys" -Force
Remove-Item "C:\Users\raph6\.ssh\authorized_keys" -Force
Remove-WindowsCapability -Online -Name "OpenSSH.Server~~~~0.0.1.0" | Out-Null
"fini" | Set-Content "C:\Users\raph6\nettoyage_fini.txt"
schtasks /delete /tn NettoyageFinal /f | Out-Null
Remove-Item $PSCommandPath -Force
