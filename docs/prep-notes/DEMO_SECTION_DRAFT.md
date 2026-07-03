# 60-second demo (draft, needs verification before use)

Drafted by mmx/MiniMax, grounded in verified facts (CI shows 171/172
tests passing, real README-documented commands). Two things need
checking before this gets used anywhere:

1. **Fixed a hallucinated URL** — mmx's first draft invented
   `https://pincher.dev/install.sh`, which doesn't match the real
   install command in the repo's actual README
   (`curl -fsSL https://raw.githubusercontent.com/SuperInstance/pincher/main/install.sh | bash`).
   Corrected below.
2. **`echo "..." | pincher teach` is an unverified guess**, not
   confirmed — `pincher teach` is documented as "interactive," and
   whether it actually accepts piped stdin this way was never checked
   (no Rust toolchain available in this environment to test it, and
   this wasn't in scope for aider's read-only investigation either).
   Don't use this demo section as-is until someone with a working
   toolchain confirms this actually works, or until aider's/another
   pass's source reading confirms the teach command's stdin behavior.

---

Pincher embeds your intents into a 384-dim vector space, fires known
reflexes in under 50ms with zero LLM calls, and learns from every miss.

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/SuperInstance/pincher/main/install.sh | bash

# Health check
pincher status

# Teach a reflex (interactive — see caveat above on piping this)
echo "show disk usage" | pincher teach

# Run it — fires in <50ms, no LLM needed
pincher do "show disk usage"
```
