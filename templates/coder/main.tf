terraform {
  required_providers {
    coder = {
      source = "coder/coder"
    }
    docker = {
      source = "kreuzwerker/docker"
    }
  }
}

provider "coder" {}
provider "docker" {}

data "coder_provisioner" "me" {}
data "coder_workspace" "me" {}
data "coder_workspace_owner" "me" {}

# --- Parameters ---

data "coder_parameter" "repo_url" {
  name         = "repo_url"
  display_name = "Repository URL"
  description  = "Git repository to clone into the workspace"
  type         = "string"
  mutable      = false
  default      = ""
}

data "coder_parameter" "agent_type" {
  name         = "agent_type"
  display_name = "Agent Type"
  description  = "Which agent runtime to use"
  type         = "string"
  mutable      = false
  default      = "claude-code"
  option {
    name  = "Claude Code"
    value = "claude-code"
  }
  option {
    name  = "Seam Agent (LangGraph)"
    value = "seam-agent"
  }
  option {
    name  = "Custom"
    value = "custom"
  }
}

data "coder_parameter" "ttl_hours" {
  name         = "ttl_hours"
  display_name = "TTL (hours)"
  description  = "Auto-stop workspace after this many hours of inactivity"
  type         = "number"
  mutable      = true
  default      = "2"
}

# --- Agent ---

resource "coder_agent" "main" {
  arch = data.coder_provisioner.me.arch
  os   = "linux"
  dir  = "/workspace"

  display_apps {
    vscode     = true
    web_terminal = true
  }

  metadata {
    display_name = "CPU"
    key          = "cpu"
    script       = "top -bn1 | awk '/Cpu/ {print $2}'"
    interval     = 10
  }

  metadata {
    display_name = "Memory"
    key          = "mem"
    script       = "free -m | awk '/Mem:/ {printf \"%.0f%%\", $3/$2*100}'"
    interval     = 10
  }
}

# Startup script: install tools and clone repo
resource "coder_script" "setup" {
  agent_id     = coder_agent.main.id
  display_name = "Agent Setup"
  icon         = "/icon/coder.svg"
  run_on_start = true
  start_blocks_login = true
  script       = templatefile("${path.module}/scripts/setup.sh", {
    repo_url   = data.coder_parameter.repo_url.value
    agent_type = data.coder_parameter.agent_type.value
    coder_url  = data.coder_workspace.me.access_url
  })
}

# Coder MCP server inside workspace for agent self-management
resource "coder_script" "mcp_server" {
  agent_id     = coder_agent.main.id
  display_name = "Coder MCP Server"
  run_on_start = true
  script       = <<-EOT
    # Start Coder MCP server so agents inside can manage workspaces
    if command -v coder &>/dev/null; then
      echo "Coder CLI available, MCP server can be started by agents"
    fi
  EOT
}

# App for the agent UI (used by Coder Tasks)
resource "coder_app" "agent" {
  agent_id     = coder_agent.main.id
  slug         = "agent"
  display_name = "Agent"
  icon         = "/icon/coder.svg"
  command      = "/workspace/bin/start-agent.sh"
}

# Make this template task-capable
resource "coder_ai_task" "task" {
  app_id = coder_app.agent.id
}

# --- Container ---

resource "docker_volume" "workspace" {
  name = "coder-${data.coder_workspace.me.id}-workspace"
  lifecycle {
    ignore_changes = all
  }
}

resource "docker_image" "workspace" {
  name = "codercom/enterprise-base:ubuntu"
}

resource "docker_container" "workspace" {
  count    = data.coder_workspace.me.start_count
  name     = "coder-${data.coder_workspace_owner.me.name}-${lower(data.coder_workspace.me.name)}"
  image    = docker_image.workspace.name
  hostname = data.coder_workspace.me.name

  entrypoint = ["sh", "-c", replace(coder_agent.main.init_script, "/localhost|127\\.0\\.0\\.1/", "host.docker.internal")]

  env = [
    "CODER_AGENT_TOKEN=${coder_agent.main.token}",
  ]

  host {
    host = "host.docker.internal"
    ip   = "host-gateway"
  }

  volumes {
    container_path = "/workspace"
    volume_name    = docker_volume.workspace.name
    read_only      = false
  }
}
