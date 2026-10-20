param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Arguments,
    [string]$Etiquette = "essai",
    [int]$DelaiMax = 900,
    [string]$Journal = ""
)

if ($Journal -ne "") {
    $proc = Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru -NoNewWindow -WorkingDirectory (Split-Path $Exe) -RedirectStandardOutput "$Journal.out" -RedirectStandardError "$Journal.err"
} else {
    $proc = Start-Process -FilePath $Exe -ArgumentList $Arguments -PassThru -NoNewWindow -WorkingDirectory (Split-Path $Exe)
}
$debut = Get-Date
$picVram = 0
$picRam = 0
$expire = $false
while (-not $proc.HasExited) {
    if (((Get-Date) - $debut).TotalSeconds -gt $DelaiMax) { $expire = $true; Stop-Process -Id $proc.Id -Force; break }
    try {
        $c = Get-Counter -Counter "\GPU Process Memory(pid_$($proc.Id)_*)\Dedicated Usage" -ErrorAction Stop
        $v = ($c.CounterSamples | Measure-Object -Property CookedValue -Sum).Sum / 1MB
        if ($v -gt $picVram) { $picVram = $v }
    } catch {}
    try {
        $proc.Refresh()
        $r = $proc.WorkingSet64 / 1MB
        if ($r -gt $picRam) { $picRam = $r }
    } catch {}
    Start-Sleep -Milliseconds 300
}
$proc.WaitForExit()
$duree = ((Get-Date) - $debut).TotalSeconds
"RESULTAT {0};code={1};expire={5};duree_s={2:N1};vram_dediee_pic_mo={3:N0};ram_pic_mo={4:N0}" -f $Etiquette, $proc.ExitCode, $duree, $picVram, $picRam, $expire
