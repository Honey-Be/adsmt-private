# adsmt project tasks

# Path to Claude Code's auto-memory dir for this project.
# Derived from the absolute project path by replacing `/` with `-`,
# matching the encoding Claude Code uses under ~/.claude/projects/.
_memory_dir := env_var("HOME") + "/.claude/projects/" + replace(justfile_directory(), "/", "-") + "/memory"

# Latest session UUID is mirrored to a stable path for resume helpers.
_session_file := justfile_directory() + "/.claude-latest-session-id"

# Default: list available recipes.
default:
    @just --list

# Mirror Claude Code's auto-memory into .claude-memories/ for version control.
# Safe to re-run; uses rsync so updates are incremental.
mirror-memory:
    @mkdir -p .claude-memories
    @if [ -d "{{_memory_dir}}" ] && [ -n "$(ls -A '{{_memory_dir}}' 2>/dev/null)" ]; then \
        rsync -a --delete '{{_memory_dir}}/' .claude-memories/ ; \
        echo "✓ Mirrored {{_memory_dir}} → .claude-memories/" ; \
    else \
        echo "⚠ {{_memory_dir}} is empty or missing — nothing to mirror" ; \
    fi

# Restore Claude Code memory from .claude-memories/ and print resume hint.
# Use after a system update or fresh checkout to pick up where you left off.
claude-resume:
    @echo "=== adsmt: Claude Code resume helper ==="
    @echo ""
    @if [ -d .claude-memories ] && [ -n "$(ls -A .claude-memories 2>/dev/null)" ]; then \
        mkdir -p '{{_memory_dir}}' ; \
        rsync -a .claude-memories/ '{{_memory_dir}}/' ; \
        echo "✓ Restored .claude-memories/ → {{_memory_dir}}" ; \
    else \
        echo "⚠ .claude-memories/ empty or missing — nothing to restore" ; \
    fi
    @echo ""
    @latest=$(ls -t .claude-conversations/*.md 2>/dev/null | head -1); \
    if [ -n "$latest" ]; then \
        echo "Latest design conversation: $latest" ; \
        echo "  Size: $(wc -l < "$latest") lines" ; \
    else \
        echo "No .claude-conversations/ logs found." ; \
    fi
    @echo ""
    @if [ -f "{{_session_file}}" ]; then \
        echo "Last recorded session: $(cat '{{_session_file}}')" ; \
    fi
    @echo ""
    @echo "To resume context, start Claude Code in this directory and ask:"
    @echo "  \"Read the latest file in .claude-conversations/ and continue.\""
