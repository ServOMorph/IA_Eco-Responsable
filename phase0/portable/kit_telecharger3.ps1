$r = "C:\TestZImage"
curl.exe -L --fail -C - -s -o "$r\modeles\z_image_turbo-Q8_0.gguf" "https://huggingface.co/leejet/Z-Image-Turbo-GGUF/resolve/main/z_image_turbo-Q8_0.gguf"
"{0} code={1} Q8_0" -f (Get-Date -Format "HH:mm:ss"), $LASTEXITCODE | Add-Content "$r\dl3.log"
"TOUT_FINI3" | Add-Content "$r\dl3.log"
