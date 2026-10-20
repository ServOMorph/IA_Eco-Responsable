#Requires -RunAsAdministrator
$ErrorActionPreference = "Stop"

$cle = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJJfAZJEvoMdicL2+JMSDF/AoC97+1RJ5FIXIjV3Bb+v test-portable-gtx1650ti"

$installe = Get-WindowsCapability -Online -Name "OpenSSH.Server*" | Where-Object State -eq "Installed"
if (-not $installe) {
    Add-WindowsCapability -Online -Name "OpenSSH.Server~~~~0.0.1.0" | Out-Null
}

$conf = "$env:ProgramData\ssh\sshd_config"
Start-Service sshd
Stop-Service sshd
(Get-Content $conf) |
    Where-Object { $_ -notmatch "^\s*PasswordAuthentication\s" } |
    Set-Content $conf -Encoding ascii
Add-Content $conf "`r`nPasswordAuthentication no"

$fichierAdmin = "$env:ProgramData\ssh\administrators_authorized_keys"
Set-Content -Path $fichierAdmin -Value $cle -Encoding ascii
icacls $fichierAdmin /inheritance:r /grant "*S-1-5-32-544:F" /grant "*S-1-5-18:F" | Out-Null

$dossier = "$env:USERPROFILE\.ssh"
New-Item -ItemType Directory -Force $dossier | Out-Null
$fichierUser = "$dossier\authorized_keys"
Set-Content -Path $fichierUser -Value $cle -Encoding ascii
icacls $fichierUser /inheritance:r /grant "*S-1-5-18:F" /grant "${env:USERNAME}:F" | Out-Null

Restart-Service sshd -ErrorAction SilentlyContinue
Write-Host "Compte Windows : $(whoami)"

Disable-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -ErrorAction SilentlyContinue
if (-not (Get-NetFirewallRule -Name "TestSSH-LAN" -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -Name "TestSSH-LAN" -DisplayName "Test SSH reseau local" -Enabled True `
        -Direction Inbound -Protocol TCP -LocalPort 22 -Action Allow -RemoteAddress LocalSubnet -Profile Any | Out-Null
}

Set-Service sshd -StartupType Manual
Start-Service sshd

$ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -like "192.168.*" -and $_.PrefixOrigin -ne "WellKnown" } | Select-Object -First 1).IPAddress
Write-Host ""
Write-Host "SSH actif (cle uniquement, reseau local uniquement)."
Write-Host "Nom d'utilisateur : $env:USERNAME"
Write-Host "Adresse IP        : $ip"
