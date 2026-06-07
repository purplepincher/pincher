"""Entry point for pincher-infer.

Usage:
    python -m pincher_infer --socket /tmp/pincher.sock
    python -m pincher_infer  # defaults to /tmp/pincher.sock
"""

from __future__ import annotations

import sys

from .server import main


if __name__ == "__main__":
    main()
