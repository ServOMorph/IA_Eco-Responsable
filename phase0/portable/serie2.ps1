$r = "C:\TestZImage"
$res = "$r\resultats2.txt"
$prompt = "portrait photo of an elderly fisherman with deep wrinkles and a grey beard, golden hour light"

function Essai($etiquette, $moteur, $extra, $larg, $haut) {
    $exe = "$r\moteurs\$moteur\sd-cli.exe"
    $a = "--diffusion-model $r\modeles\z_image_turbo-Q3_K.gguf --vae $r\modeles\ae.safetensors --llm $r\modeles\Qwen3-4B-Instruct-2507-Q4_K_M.gguf -p `"$prompt`" --cfg-scale 1.0 --steps 8 -H $haut -W $larg -s 42 $extra -o $r\sortie\$etiquette.png"
    $o = & "$r\mesure.ps1" -Exe $exe -Arguments $a -Etiquette $etiquette -DelaiMax 900 -Journal "$r\sortie\$etiquette" 2>&1 | Out-String
    ($o -split "`n" | Where-Object { $_ -match "^RESULTAT" }) | Add-Content $res
    $txt = (Get-Content "$r\sortie\$etiquette.err", "$r\sortie\$etiquette.out" -ErrorAction SilentlyContinue) -join "`n"
    ($txt -split "[`r`n]+" | Where-Object { $_ -match "completed|taking|Device|backend" }) | ForEach-Object { "   " + $_.Substring(0, [Math]::Min(150, $_.Length)) } | Add-Content $res
}

"debut serie2 $(Get-Date -Format HH:mm:ss)" | Add-Content $res
Essai "cuda_512_run2" "cuda" "--offload-to-cpu --vae-tiling" 512 512
Essai "cuda_512_nooff" "cuda" "--vae-tiling" 512 512
Essai "vulkan_512_off" "vulkan" "--offload-to-cpu --vae-tiling" 512 512
"FIN_SERIE2 $(Get-Date -Format HH:mm:ss)" | Add-Content $res
