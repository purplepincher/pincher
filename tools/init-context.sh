#!/usr/bin/env bash
# init-context.sh — Initialize CONTEXT.md for a new session
#
# Called at session cold-start. Creates or refreshes CONTEXT.md.
# Archives the previous CONTEXT.md to memory/archive/CONTEXT-YYYY-MM-DD.md.
#
# The split: MEMORY.md is immortal. CONTEXT.md is ephemeral.
# This script enforces that boundary.

set -euo pipefail

WORKSPACE="/home/ubuntu/.openclaw/workspace"
CONTEXT_FILE="${WORKSPACE}/library/CONTEXT.md"
CONTEXT_ARCHIVE="${WORKSPACE}/memory/archive"
TIMESTAMP=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
DATE=$(date -u '+%Y-%m-%d')

mkdir -p "${CONTEXT_ARCHIVE}"

# Archive old CONTEXT.md if it exists
if [ -f "${CONTEXT_FILE}" ]; then
    cp "${CONTEXT_FILE}" "${CONTEXT_ARCHIVE}/CONTEXT-${DATE}.md"
    echo "📦 Archived previous CONTEXT.md → ${CONTEXT_ARCHIVE}/CONTEXT-${DATE}.md"
fi

# Write fresh CONTEXT.md
cat > "${CONTEXT_FILE}" <<CONTEXT
# CONTEXT.md — Session-Relevant Context
# Initialized: ${TIMESTAMP}
# This file is ephemeral. Immortal facts go in MEMORY.md.
# CONTEXT.md is archived to memory/archive/ on each session start.

## Active Tasks
- (set during session)

## Recent State Changes
- Session initialized at ${TIMESTAMP}

## Reflex Fire Timestamps
| Reflex | Last Fired | Expected Window |
|--------|-----------|-----------------|

## Blocker/Alert State
- No blockers
- No critical alerts
CONTEXT

echo "✅ Fresh CONTEXT.md written"
echo ""
echo "📋 Reminder:"
echo "   MEMORY.md = immortal facts (user info, fleet, protocols)"
echo "   CONTEXT.md = session state (current tasks, recent decisions)"
echo "   Rule: If it won't matter in 48 hours, put it in CONTEXT.md"
