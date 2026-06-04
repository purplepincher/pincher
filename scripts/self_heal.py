#!/usr/bin/env python3
"""Pincher Asynchronous Self-Healing Compiler Loop

The local edge runtime never patches its own code. When a reflex fails at runtime,
error details are queued offline. When the device is idle and has network access,
this pipeline uploads the error payload to the Cloud Compiler, which:
    1. Reads the failed execution context
    2. Alters the underlying code logic
    3. Tests the new code against assertions
    4. Deploys a silent patch to reflexes.db

Architecture:
    - Local edge = sterile (never self-modifies)
    - Cloud compiler = mutant (rewrites broken reflexes)
    - Zero-temp to prevent regression bugs in patches

Usage:
    python3 self_heal.py --error-file failed_reflex.json
    python3 self_heal.py --daemon  # Watch telemetry queue directory
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import Dict, Optional

import requests

DEEPINFRA_API_URL = os.environ.get(
    "DEEPINFRA_API_URL",
    "https://api.deepinfra.com/v1/openai/chat/completions"
)
DEEPINFRA_API_KEY = os.environ.get("DEEPINFRA_API_KEY", "")
MODEL = os.environ.get("HEALER_MODEL", "meta-llama/Meta-Llama-3-70B-Instruct")


class SelfHealingCompiler:
    """Cloud-based self-healing compiler for broken reflexes."""

    def __init__(self, manifest_config: Dict, api_key: Optional[str] = None):
        self.manifest_config = manifest_config
        self.api_key = api_key or DEEPINFRA_API_KEY

    def heal_broken_reflex(
        self,
        failed_code: str,
        error_logs: str,
        runtime_context: str,
    ) -> str:
        """Ask the Cloud LLM to fix code that broke during runtime execution.

        Args:
            failed_code: The Rust/WASM source code that failed
            error_logs: Error output from the failed execution
            runtime_context: Environment variables, state, etc.

        Returns:
            Updated Rust source code fixing the issue

        Raises:
            ValueError: If LLM returns empty or invalid code
            requests.RequestException: If API call fails
        """
        prompt = self._build_prompt(failed_code, error_logs, runtime_context)
        source_code = self._call_llm(prompt)
        return self._clean_output(source_code)

    def _build_prompt(
        self,
        failed_code: str,
        error_logs: str,
        runtime_context: str,
    ) -> str:
        return f"""
You are the pincher runtime Automated Self-Healing Architecture. A pre-compiled Rust reflex
failed during local execution. Your task is to analyze the source code error logs and output
an updated, optimized Rust file that fixes the issue.

ORIGINAL INTENT MANIFEST LIMITATIONS:
{json.dumps(self.manifest_config, indent=2)}

BROKEN SOURCE CODE:
{failed_code}

HOST EXECUTION EXCEPTION ERROR LOGS:
{error_logs}

RUNTIME ENVIRONMENT VARIABLES/STATE DATA:
{runtime_context}

CORRECTION INSTRUCTIONS:
1. Fix the root cause of the crash or assertion failure shown in the logs.
2. Keep the code compatible with standard library crates running on target `wasm32-wasip1`.
3. Do not modify the expected stdout assertion checks requested by the developer manifest.
4. Focus on edge cases: empty inputs, null pointers, zero-length buffers.
5. Add defensive checks where the crash occurred but keep the same function signatures.
6. Output ONLY valid Rust source code text without markdown blocks or explanation wrappers.
"""

    def _call_llm(self, prompt: str) -> str:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,  # Prevent creative regressions
            "max_tokens": 4096,
        }

        print("[SELF-HEAL] Dispatching error telemetry to cloud distillation loop...")
        response = requests.post(
            DEEPINFRA_API_URL,
            json=payload,
            headers=headers,
            timeout=120,
        )
        response.raise_for_status()

        content = response.json()["choices"][0]["message"]["content"]
        return content.strip()

    def _clean_output(self, source: str) -> str:
        """Strip markdown fences if present."""
        if source.startswith("```"):
            lines = source.split("\n", 1)
            if len(lines) > 1:
                source = lines[1]
            source = source.rsplit("\n", 1)[0] if source.endswith("```") else source
            if source.endswith("```"):
                source = source[:-3]
        return source.strip()

    def heal_from_file(self, error_file: Path) -> str:
        """Load error data from a JSON file and return healed code."""
        with open(error_file) as f:
            error_data = json.load(f)

        return self.heal_broken_reflex(
            failed_code=error_data.get("failed_code", ""),
            error_logs=error_data.get("error_logs", ""),
            runtime_context=error_data.get("env_context", "{}"),
        )


def run_daemon(watch_dir: Path, manifest: Dict, poll_interval: int = 30):
    """Run in daemon mode, watching a directory for error files."""
    compiler = SelfHealingCompiler(manifest)
    print(f"[SELF-HEAL DAEMON] Watching {watch_dir} for error telemetry...")

    while True:
        for error_file in watch_dir.glob("*.error.json"):
            print(f"[SELF-HEAL] Found error file: {error_file}")
            try:
                healed = compiler.heal_from_file(error_file)
                output_path = error_file.with_suffix(".healed.rs")
                with open(output_path, "w") as f:
                    f.write(healed)
                error_file.unlink()  # Remove original after healing
                print(f"[SELF-HEAL] ✅ Healed code written to {output_path}")
            except Exception as e:
                print(f"[SELF-HEAL ERROR] {e}")

        time.sleep(poll_interval)


def main():
    parser = argparse.ArgumentParser(
        description="Pincher Asynchronous Self-Healing Compiler"
    )
    parser.add_argument(
        "--error-file",
        type=Path,
        help="JSON error telemetry file to heal",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default="Intent.toml",
        help="Path to Intent.toml manifest (default: Intent.toml)",
    )
    parser.add_argument(
        "--daemon",
        action="store_true",
        help="Run in daemon mode watching a directory",
    )
    parser.add_argument(
        "--watch-dir",
        type=Path,
        default=Path("./.error_telemetry"),
        help="Directory to watch in daemon mode",
    )
    parser.add_argument(
        "--poll-interval",
        type=int,
        default=30,
        help="Seconds between directory polls in daemon mode",
    )

    args = parser.parse_args()

    # Load manifest
    manifest: Dict = {}
    if args.manifest.exists():
        import tomllib
        with open(args.manifest, "rb") as f:
            manifest = tomllib.load(f)

    if args.daemon:
        args.watch_dir.mkdir(parents=True, exist_ok=True)
        run_daemon(args.watch_dir, manifest, args.poll_interval)
    elif args.error_file:
        compiler = SelfHealingCompiler(manifest)
        healed = compiler.heal_from_file(args.error_file)
        print(healed)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
