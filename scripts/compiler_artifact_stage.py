# 📄 Dosya Yolu: /turkuazvm/scripts/compiler_artifact_stage.py
# 📌 Amac: GitHub Actions indirilen compiler artifactlarini gate'in bekledigi canonical dizine fail-open olmadan tasir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Eksik platformu uydurmaz; bulunan normalized JSON dosyalarini merkezi config yollarina gore stage eder
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
from pathlib import Path
import shutil

from compiler_gate_policy import PolicyError, artifact_value, load_policy, required_platforms


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", default=None)
    parser.add_argument("--output-dir", default=None)
    parser.add_argument("--config", default=None)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]

    try:
        policy = load_policy(root, args.config)
    except PolicyError as error:
        print(f"COMPILER ARTIFACT STAGE: BLOCKED {error.code} {error.detail}")
        return 2

    input_root = Path(args.input_dir) if args.input_dir else Path(artifact_value(policy, "downloaded_root"))
    output_root = Path(args.output_dir) if args.output_dir else Path(artifact_value(policy, "normalized_root"))
    if not input_root.is_absolute():
        input_root = root / input_root
    if not output_root.is_absolute():
        output_root = root / output_root

    summary_name = artifact_value(policy, "normalized_summary")
    errors_name = artifact_value(policy, "normalized_errors")
    artifact_prefix = artifact_value(policy, "artifact_name_prefix")
    artifact_root = Path(artifact_value(policy, "raw_root")).parent
    normalized_relative = Path(artifact_value(policy, "normalized_root")).relative_to(artifact_root)

    staged = 0
    for platform_name in required_platforms(policy):
        candidates = [
            input_root / f"{artifact_prefix}{platform_name}" / normalized_relative / platform_name,
            input_root / normalized_relative / platform_name,
        ]
        source = next((candidate for candidate in candidates if candidate.is_dir()), None)
        if source is None:
            print(f"COMPILER ARTIFACT STAGE: MISSING platform={platform_name}")
            continue
        destination = output_root / platform_name
        destination.mkdir(parents=True, exist_ok=True)
        for file_name in (summary_name, errors_name):
            source_file = source / file_name
            if source_file.is_file():
                shutil.copy2(source_file, destination / file_name)
        staged += 1
        print(f"COMPILER ARTIFACT STAGE: STAGED platform={platform_name}")

    print(f"COMPILER ARTIFACT STAGE: COMPLETE staged={staged}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
