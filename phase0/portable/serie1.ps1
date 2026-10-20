$r = "C:\TestZImage"
$res = "$r\resultats.txt"
$prompt = "portrait photo of an elderly fisherman with deep wrinkles and a grey beard, golden hour light"

function Essai($etiquette, $moteur, $extra, $larg, $haut) {
    $exe = "$r\moteurs\$moteur\sd-cli.exe"
    $a = "--diffusion-model $r\modeles\z_image_turbo-Q3_K.gguf --vae $r\modeles\ae.safetensors --llm $r\modeles\Qwen3-4B-Instruct-2507-Q4_K_M.gguf -p `"$prompt`" --cfg-scale 1.0 --steps 8 -H $haut -W $larg -s 42 $extra -o $r\sortie\$etiquette.png"
    $o = & "$r\mesure.ps1" -Exe $exe -Arguments $a -Etiquette $etiquette -DelaiMax 900 2>&1 | Out-String
    $o | Set-Content "$r\sortie\$etiquette.log"
    ($o -split "`n" | Where-Object { $_ -match "^RESULTAT" }) | Add-Content $res
}

New-Item -ItemType Directory -Force "$r\sortie" | Out-Null
"debut serie1 $(Get-Date -Format HH:mm:ss)" | Add-Content $res
Essai "cuda_512_off" "cuda" "--offload-to-cpu --vae-tiling" 512 512
Essai "cuda_768_off" "cuda" "--offload-to-cpu --vae-tiling" 768 768
Essai "cuda_1024_off" "cuda" "--offload-to-cpu --vae-tiling" 1024 1024
"FIN_SERIE1 $(Get-Date -Format HH:mm:ss)" | Add-Content $res
