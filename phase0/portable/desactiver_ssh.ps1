#Requires -RunAsAdministrator
$ErrorActionPreference = "SilentlyContinue"

Stop-Service sshd
Set-Service sshd -StartupType Disabled
Remove-NetFirewallRule -Name "TestSSH-LAN"
Remove-Item "$env:ProgramData\ssh\administrators_authorized_keys" -Force
Remove-Item "$env:USERPROFILE\.ssh\authorized_keys" -Force

Write-Host "SSH desactive, regle pare-feu supprimee, cle retiree."
