param(
    [Parameter(Mandatory = $true)][string]$Tests,
    [string]$Sortie = "C:\TestZImage\resultats.jsonl"
)

$r = "C:\TestZImage"
$promptDefaut = "portrait photo of an elderly fisherman with deep wrinkles and a grey beard, golden hour light"
$liste = Get-Content $Tests -Raw | ConvertFrom-Json
New-Item -ItemType Directory -Force "$r\sortie" | Out-Null

function Telemetrie() {
    try {
        $l = & nvidia-smi --query-gpu=clocks.sm,power.draw,temperature.gpu,clocks_throttle_reasons.active,memory.used --format=csv,noheader,nounits 2>$null
        $p = $l.Split(",") | ForEach-Object { $_.Trim() }
        return @{ clock = [double]$p[0]; power = [double]$p[1]; temp = [double]$p[2]; raisons = $p[3]; vram = [double]$p[4] }
    } catch { return $null }
}

foreach ($t in $liste) {
    $label = $t.label
    $moteur = if ($t.moteur) { $t.moteur } else { "cuda" }
    $diff = if ($t.modele) { $t.modele } else { "z_image_turbo-Q3_K.gguf" }
    $te = if ($t.te) { $t.te } else { "Qwen3-4B-Instruct-2507-Q4_K_M.gguf" }
    $w = if ($t.w) { $t.w } else { 512 }
    $h = if ($t.h) { $t.h } else { 512 }
    $steps = if ($t.steps) { $t.steps } else { 8 }
    $seed = if ($t.seed) { $t.seed } else { 42 }
    $extra = if ($t.extra) { $t.extra } else { "" }
    $prompt = if ($t.prompt) { $t.prompt } else { $promptDefaut }
    $delai = if ($t.delai) { $t.delai } else { 1500 }
    $exe = "$r\moteurs\$moteur\sd-cli.exe"
    $a = "--diffusion-model $r\modeles\$diff --vae $r\modeles\ae.safetensors --llm $r\modeles\$te -p `"$prompt`" --cfg-scale 1.0 --steps $steps -H $h -W $w -s $seed $extra -v -o $r\sortie\$label.png"

    $proc = Start-Process -FilePath $exe -ArgumentList $a -PassThru -NoNewWindow -WorkingDirectory (Split-Path $exe) -RedirectStandardOutput "$r\sortie\$label.out" -RedirectStandardError "$r\sortie\$label.err"
    $debut = Get-Date
    $picVram = 0; $picRam = 0; $expire = $false
    $clocks = @(); $tempMax = 0; $powerMax = 0; $raisons = @{}; $vramGpuMax = 0
    $dernier = Get-Date "2000-01-01"
    while (-not $proc.HasExited) {
        if (((Get-Date) - $debut).TotalSeconds -gt $delai) { $expire = $true; Stop-Process -Id $proc.Id -Force; break }
        try {
            $c = Get-Counter -Counter "\GPU Process Memory(pid_$($proc.Id)_*)\Dedicated Usage" -ErrorAction Stop
            $v = ($c.CounterSamples | Measure-Object -Property CookedValue -Sum).Sum / 1MB
            if ($v -gt $picVram) { $picVram = $v }
        } catch {}
        try { $proc.Refresh(); $rr = $proc.WorkingSet64 / 1MB; if ($rr -gt $picRam) { $picRam = $rr } } catch {}
        if (((Get-Date) - $dernier).TotalSeconds -ge 3) {
            $tm = Telemetrie
            if ($tm) {
                $clocks += $tm.clock
                if ($tm.temp -gt $tempMax) { $tempMax = $tm.temp }
                if ($tm.power -gt $powerMax) { $powerMax = $tm.power }
                if ($tm.vram -gt $vramGpuMax) { $vramGpuMax = $tm.vram }
                $raisons[$tm.raisons] = 1
            }
            $dernier = Get-Date
        }
        Start-Sleep -Milliseconds 300
    }
    $proc.WaitForExit()
    $duree = ((Get-Date) - $debut).TotalSeconds
    $actifs = $clocks | Where-Object { $_ -gt 600 }
    $res = [ordered]@{
        label = $label; moteur = $moteur; modele = $diff; te = $te; w = $w; h = $h; steps = $steps; extra = $extra
        duree_s = [math]::Round($duree, 1); expire = $expire; code = $proc.ExitCode
        vram_dediee_pic_mo = [math]::Round($picVram); ram_pic_mo = [math]::Round($picRam); vram_gpu_pic_mo = $vramGpuMax
        horloge_moy_mhz = if ($actifs) { [math]::Round(($actifs | Measure-Object -Average).Average) } else { 0 }
        horloge_min_mhz = if ($actifs) { ($actifs | Measure-Object -Minimum).Minimum } else { 0 }
        temp_max_c = $tempMax; puissance_max_w = $powerMax; raisons_bridage = ($raisons.Keys -join "|")
    }
    ($res | ConvertTo-Json -Compress) | Add-Content $Sortie
}
"FIN $(Split-Path $Tests -Leaf)" | Add-Content "$Sortie.fin"
