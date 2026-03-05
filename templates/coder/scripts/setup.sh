#!/usr/bin/env bash
# Workspace setup script — runs on start, blocks login until complete.
# Template variables: repo_url, agent_type, coder_url
set -euo pipefail

echo "=== Agent Workspace Setup ==="

# --- Ensure workspace volume is owned by current user ---
sudo chown -R "$(id -u):$(id -g)" /workspace

# --- Core tools ---
sudo apt-get update -qq
sudo apt-get install -y -qq git curl jq unzip > /dev/null 2>&1

# --- Clone repo if specified ---
if [ -n "${repo_url}" ]; then
  if [ ! -d "/workspace/repo/.git" ]; then
    echo "Cloning ${repo_url}..."
    git clone "${repo_url}" /workspace/repo
  else
    echo "Repo already cloned, pulling latest..."
    cd /workspace/repo && git pull --ff-only || true
  fi
fi

# --- Install agent runtime based on type ---
mkdir -p /workspace/bin

case "${agent_type}" in
  claude-code)
    echo "Installing Claude Code..."
    if ! command -v claude &>/dev/null; then
      curl -fsSL https://cli.claude.ai/install.sh | sh
    fi
    # Create agent start script
    cat > /workspace/bin/start-agent.sh << 'AGENT_EOF'
#!/usr/bin/env bash
cd /workspace/repo 2>/dev/null || cd /workspace
exec claude --dangerously-skip-permissions
AGENT_EOF
    chmod +x /workspace/bin/start-agent.sh
    ;;

  seam-agent)
    echo "Installing Seam Agent runtime..."
    # Install uv for Python dependency management
    if ! command -v uv &>/dev/null; then
      curl -LsSf https://astral.sh/uv/install.sh | sh
      export PATH="$HOME/.local/bin:$PATH"
    fi
    # Install seam-agents if pyproject.toml exists
    if [ -f "/workspace/repo/pyproject.toml" ]; then
      cd /workspace/repo && uv sync
    fi
    cat > /workspace/bin/start-agent.sh << 'AGENT_EOF'
#!/usr/bin/env bash
cd /workspace/repo 2>/dev/null || cd /workspace
exec uv run seam-agent "$@"
AGENT_EOF
    chmod +x /workspace/bin/start-agent.sh
    ;;

  custom)
    echo "Custom agent type — no automatic setup."
    cat > /workspace/bin/start-agent.sh << 'AGENT_EOF'
#!/usr/bin/env bash
echo "Custom agent: configure /workspace/bin/start-agent.sh"
AGENT_EOF
    chmod +x /workspace/bin/start-agent.sh
    ;;
esac

# --- Install tackline skills (if available) ---
if [ -d "/workspace/repo/tackline" ]; then
  echo "Installing tackline..."
  cd /workspace/repo/tackline && bash install.sh || true
elif command -v git &>/dev/null; then
  # Clone tackline for skills/agents
  if [ ! -d "/workspace/.tackline" ]; then
    git clone https://github.com/tacklines/tackline.git /workspace/.tackline 2>/dev/null || true
    if [ -d "/workspace/.tackline" ]; then
      cd /workspace/.tackline && bash install.sh || true
    fi
  fi
fi

# --- Coder CLI config ---
if command -v coder &>/dev/null; then
  coder config-ssh --yes 2>/dev/null || true
fi

echo "=== Setup complete ==="
