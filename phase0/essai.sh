#!/bin/bash
# usage: essai.sh etiquette moteur(cuda|vulkan) "options supplementaires" [diffusion.gguf]
ET="$1"; MOT="$2"; EXTRA="$3"; DIF="${4:-D:\ServOMorph\ComfyUI-Qwen\models\unet\z_image_turbo-Q4_K_M.gguf}"
M='D:\ServOMorph\IA_Eco-Responsable\phase0\modeles'; C='D:\ServOMorph\ComfyUI-Qwen\models'
E="D:\ServOMorph\IA_Eco-Responsable\phase0\moteurs\$MOT\sd-cli.exe"
ARGS="--diffusion-model $DIF --vae $C\vae\z_image_ae.safetensors --llm $M\Qwen3-4B-Instruct-2507-Q4_K_M.gguf -p \"portrait photo of an elderly fisherman with deep wrinkles and a grey beard, golden hour light\" --cfg-scale 1.0 --steps 8 -H 1024 -W 1024 -s 42 $EXTRA -o D:\ServOMorph\IA_Eco-Responsable\phase0\sortie\$ET.png"
powershell -NoProfile -ExecutionPolicy Bypass -File "$(dirname "$0")/mesure.ps1" -Exe "$E" -Arguments "$ARGS" -Etiquette "$ET" 2>&1 | tr '\r' '\n' | grep -E "^$ET;|completed|ERROR|\[E\]|assert|failed" | cut -c1-200
