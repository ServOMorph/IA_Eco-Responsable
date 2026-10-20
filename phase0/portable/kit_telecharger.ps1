$r = "C:\TestZImage"
New-Item -ItemType Directory -Force "$r\moteurs", "$r\modeles", "$r\sortie" | Out-Null
$log = "$r\dl.log"
$b = "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-973-f0aac52"

function Dl($url, $dest) {
    curl.exe -L --fail -C - --retry 5 -s -o $dest $url
    "{0} code={1} {2}" -f (Get-Date -Format "HH:mm:ss"), $LASTEXITCODE, $dest | Add-Content $log
}

Dl "$b/sd-master-f0aac52-bin-win-cuda12-x64.zip" "$r\moteurs\cuda.zip"
Dl "$b/cudart-sd-bin-win-cu12-x64.zip" "$r\moteurs\cudart.zip"
Dl "$b/sd-master-f0aac52-bin-win-vulkan-x64.zip" "$r\moteurs\vulkan.zip"
Expand-Archive -Force "$r\moteurs\cuda.zip" "$r\moteurs\cuda"
Expand-Archive -Force "$r\moteurs\cudart.zip" "$r\moteurs\cuda"
Expand-Archive -Force "$r\moteurs\vulkan.zip" "$r\moteurs\vulkan"
"extraction terminee" | Add-Content $log

Dl "https://huggingface.co/leejet/Z-Image-Turbo-GGUF/resolve/main/z_image_turbo-Q3_K.gguf" "$r\modeles\z_image_turbo-Q3_K.gguf"
Dl "https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/Qwen3-4B-Instruct-2507-Q4_K_M.gguf" "$r\modeles\Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
Dl "https://huggingface.co/Comfy-Org/z_image_turbo/resolve/main/split_files/vae/ae.safetensors" "$r\modeles\ae.safetensors"
"TOUT_FINI" | Add-Content $log
