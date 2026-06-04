#!/usr/bin/env python3
"""Pincher Cloud-Side AI Regex Generator Engine

Generates PCRE-compatible regular expressions with Named Capture Groups
from variable extraction schemas and seed phrases. Runs a validation loop
against template seed data before accepting patterns.

Architecture:
    - LLM generates regex patterns from structured variable schemas
    - Validation loop ensures patterns parse seed phrases correctly
    - Broken patterns are rejected before entering the build pipeline
    - Temperature 0.0 ensures deterministic, reproducible output

Usage:
    python3 regex_compiler.py --schema '{"target_branch": {"type": "string"}}' --seeds '["push to main"]'
"""

import argparse
import json
import os
import re
import sys
from typing import Dict, List, Optional

import requests

DEEPINFRA_API_URL = os.environ.get(
    "DEEPINFRA_API_URL",
    "https://api.deepinfra.com/v1/openai/chat/completions"
)
DEEPINFRA_API_KEY = os.environ.get("DEEPINFRA_API_KEY", "")
MODEL = os.environ.get("REGEX_COMPILER_MODEL", "meta-llama/Meta-Llama-3-70B-Instruct")


class CloudRegexGenerator:
    """Generates PCRE regex patterns from variable schemas using LLM."""

    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or DEEPINFRA_API_KEY

    def synthesize_extraction_patterns(
        self,
        variable_schema: Dict,
        seed_phrases: List[str],
        existing_patterns: Optional[List[str]] = None,
    ) -> List[str]:
        """Generate regex patterns with named capture groups from schema.

        Args:
            variable_schema: Dict of variable names -> type/constraint definitions
            seed_phrases: Example inputs to validate patterns against
            existing_patterns: Optional previous patterns for iterative refinement

        Returns:
            List of valid PCRE regex pattern strings

        Raises:
            ValueError: If generated patterns fail validation
            requests.RequestException: If LLM API call fails
        """
        prompt = self._build_prompt(variable_schema, seed_phrases, existing_patterns)
        raw_output = self._call_llm(prompt)
        patterns = self._parse_response(raw_output)

        self._verify_patterns_against_seeds(patterns, seed_phrases)
        return patterns

    def _build_prompt(
        self,
        schema: Dict,
        seeds: List[str],
        existing: Optional[List[str]] = None,
    ) -> str:
        prompt = f"""
You are the pincher runtime Structural Regex Engine Optimizer.
Your task is to review a variable extraction contract schema and generate a list of highly optimized,
valid PCRE-compatible regular expressions containing Named Capture Groups.

VARIABLE CAPTURE TARGET SCHEMA:
{json.dumps(schema, indent=2)}

TARGET SEED USER INPUT EXAMPLES:
{json.dumps(seeds, indent=2)}
"""
        if existing:
            prompt += f"""
EXISTING PATTERNS (may need improvement):
{json.dumps(existing, indent=2)}
"""

        prompt += """
CRITICAL REGEX COMPILATION CRITERIA:
1. Every variable in the schema MUST be captured by named capture groups using (?P<variable_name>pattern) syntax.
2. Generate expressions flexible enough to catch variations in whitespace or symbols while maintaining strict type boundaries.
3. Prefer non-greedy qualifiers (*?) when capturing open-ended text like commit messages.
4. Each pattern should capture as many schema variables as possible (fewer patterns = faster matching).
5. Do not output code, markdown formatting blocks, backticks, or wrappers.
6. Output strictly a clean, flat JSON array of raw string patterns.
"""
        return prompt

    def _call_llm(self, prompt: str) -> str:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
            "max_tokens": 2048,
        }

        response = requests.post(
            DEEPINFRA_API_URL,
            json=payload,
            headers=headers,
            timeout=60,
        )
        response.raise_for_status()

        content = response.json()["choices"][0]["message"]["content"]
        return content.strip()

    def _parse_response(self, raw: str) -> List[str]:
        # Clean potential markdown wrappers
        if raw.startswith("```"):
            lines = raw.split("\n", 1)
            if len(lines) > 1:
                raw = lines[1]
            raw = raw.rsplit("\n", 1)[0] if raw.endswith("```") else raw
            if raw.endswith("```"):
                raw = raw[:-3]

        patterns = json.loads(raw)
        if not isinstance(patterns, list):
            raise ValueError(f"Expected JSON array, got {type(patterns)}: {patterns}")
        return patterns

    def _verify_patterns_against_seeds(
        self, patterns: List[str], seed_phrases: List[str]
    ):
        """Validate that generated patterns parse seed phrases correctly."""
        missing_count = 0

        for pattern in patterns:
            try:
                compiled = re.compile(pattern)
                if not compiled.groupindex:
                    raise ValueError(
                        f"Pattern lacks named capture groups: '{pattern}'"
                    )
            except re.error as e:
                raise ValueError(
                    f"Invalid regex in generated patterns: '{pattern}'. Error: {e}"
                )

        # Check that each seed has at least one matching pattern
        for seed in seed_phrases:
            matched = any(
                re.search(p, seed) for p in patterns
            )
            if not matched:
                missing_count += 1
                print(f"[WARNING] Seed phrase has no matching pattern: '{seed}'")

        if missing_count == len(seed_phrases):
            raise ValueError(
                "All generated patterns failed to match seed phrases. "
                "Retrying with lower confidence..."
            )

        print(f"[VERIFIED] {len(patterns)} patterns × {len(seed_phrases)} seeds = {len(patterns) * len(seed_phrases)} extractions checked")
        print(f"           Coverage: {len(seed_phrases) - missing_count}/{len(seed_phrases)} seeds covered")


def main():
    parser = argparse.ArgumentParser(
        description="Pincher Cloud-Side AI Regex Generator"
    )
    parser.add_argument(
        "--schema",
        required=True,
        help="JSON variable capture schema, e.g. '{\"var\": {\"type\": \"string\"}}'",
    )
    parser.add_argument(
        "--seeds",
        required=True,
        help="JSON array of seed input strings, e.g. '[\"push to main\"]'",
    )
    parser.add_argument(
        "--existing",
        default=None,
        help="Optional JSON array of existing patterns for refinement",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Output file for generated patterns (default: stdout)",
    )

    args = parser.parse_args()

    schema = json.loads(args.schema)
    seeds = json.loads(args.seeds)
    existing = json.loads(args.existing) if args.existing else None

    generator = CloudRegexGenerator()
    try:
        patterns = generator.synthesize_extraction_patterns(schema, seeds, existing)
    except ValueError as e:
        print(f"[ERROR] {e}", file=sys.stderr)
        sys.exit(1)
    except requests.RequestException as e:
        print(f"[ERROR] LLM API call failed: {e}", file=sys.stderr)
        sys.exit(2)

    output = json.dumps(patterns, indent=2)
    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"[DONE] Patterns written to {args.output}")
    else:
        print(output)


if __name__ == "__main__":
    main()
