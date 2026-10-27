import os
import subprocess
import sys
from pathlib import Path

RACINE_PROJET = Path(__file__).resolve().parent
EXE = RACINE_PROJET / "app" / "src-tauri" / "target" / "release" / "app.exe"
RACINE_IA = Path(r"D:\Dev\ia-eco-test")
CARGO_HOME = Path(r"D:\Dev\cargo")
RUSTUP_HOME = Path(r"D:\Dev\rustup")


def environnement():
    env = os.environ.copy()
    env.setdefault("IA_ECO_RACINE", str(RACINE_IA))
    env.setdefault("CARGO_HOME", str(CARGO_HOME))
    env.setdefault("RUSTUP_HOME", str(RUSTUP_HOME))
    env["PATH"] = str(CARGO_HOME / "bin") + os.pathsep + env["PATH"]
    return env


def compiler(env):
    subprocess.run(
        ["cargo", "build", "--release"],
        cwd=RACINE_PROJET / "app" / "src-tauri",
        env=env,
        check=True,
    )


def main():
    env = environnement()
    if "--build" in sys.argv or not EXE.exists():
        compiler(env)
    subprocess.run([str(EXE)], env=env, check=True)


if __name__ == "__main__":
    main()
