$r = "C:\TestZImage"
$log = "$r\dl2.log"
function Dl($url, $dest) {
    curl.exe -L --fail -C - --retry 5 -s -o $dest $url
    "{0} code={1} {2}" -f (Get-Date -Format "HH:mm:ss"), $LASTEXITCODE, $dest | Add-Content $log
}
$h = "https://huggingface.co/leejet/Z-Image-Turbo-GGUF/resolve/main"
foreach ($q in "Q2_K", "Q4_0", "Q4_K", "Q5_0", "Q6_K") { Dl "$h/z_image_turbo-$q.gguf" "$r\modeles\z_image_turbo-$q.gguf" }
$u = "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main"
foreach ($q in "Q3_K_M", "Q5_K_M", "Q6_K", "Q4_0", "IQ4_XS") { Dl "$u/Qwen3-4B-Instruct-2507-$q.gguf" "$r\modeles\Qwen3-4B-Instruct-2507-$q.gguf" }
"TOUT_FINI2" | Add-Content $log
