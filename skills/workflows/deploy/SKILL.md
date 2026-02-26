---
name: deploy
description: "Deploy a project to production or a target environment. Covers readiness gating, strategy selection (direct, canary, blue-green, rolling), execution, post-deployment health checks, and rollback criteria. Use when you are ready to ship and want a structured, safe deployment workflow. Keywords: deploy, ship, release, rollout, canary, blue-green, rollback, production."
argument-hint: "<service or target environment>"
disable-model-invocation: false
user-invocable: true
allowed-tools: Read, Grep, Glob, Bash(git:*), Bash(docker:*), Bash(kubectl:*), Bash(npm:*), Bash(make:*), Bash(bd:*)
context: fork
---

# Deploy: Structured Deployment Workflow

You are running the **Deploy** workflow -- a structured deployment process with readiness gating, strategy selection, execution, monitoring, and rollback criteria. Target: **$ARGUMENTS**

## When to Use

- You are ready to ship a feature and want a safe, observable deployment
- You need to choose between deployment strategies (canary, blue-green, rolling, direct)
- You want post-deployment health checks before declaring success
- You need a clear rollback plan if things go wrong

## Overview

Deploy works in 5 phases. The readiness gate (Phase 0) must pass before any deployment begins.

```
Readiness gate (tests, git, beads)
  -> Strategy selection (direct / canary / blue-green / rolling)
    -> Execution (CI/CD tooling or guided manual steps)
      -> Monitoring (health checks, logs, endpoints)
        -> Rollback criteria (automated triggers + manual guide)
```

---

## Phase 0: Readiness Gate

**This phase is a hard gate. If any check fails, stop here and report what must be fixed.**

### 0a. Detect Target

If `$ARGUMENTS` is empty, ask the user: "What are you deploying, and to which environment (staging, production, etc.)?"

If provided, confirm the target: service name and environment (e.g., "api-gateway to production").

### 0b. Git Status Check

```bash
git status --short
git log --oneline -3
```

- If there are uncommitted changes, stop. Report the dirty files. Ask the user to commit or stash before deploying.
- Confirm the current branch is the expected deploy branch (typically `main` or `release/*`).

### 0c. Test Status Check

Detect the test runner from project config:

```bash
# Check for common test configs
ls package.json Makefile pytest.ini go.mod Cargo.toml 2>/dev/null
```

- **Node.js**: `npm test` or `npm run test`
- **Python**: `pytest` or `python -m pytest`
- **Go**: `go test ./...`
- **Rust**: `cargo test`
- **Make**: `make test`

If a test command exists, report the last test run result from git log or CI artifacts if available. If tests cannot be verified, warn the user and ask for confirmation before proceeding.

### 0d. In-Progress Work Check

If `.beads/` exists:

```bash
bd query "status=open AND priority<=1"
```

If P0 or P1 work items are open and their file set overlaps with the files changed since last deploy (from `git diff --name-only HEAD~1`), warn the user: "Open high-priority work touches files in this deploy. Confirm you want to continue."

### 0e. Gate Summary

Report:

```
## Readiness Gate: [PASS | FAIL]

- Git: [clean on main | FAIL: dirty files listed]
- Tests: [verified passing | unverified — user confirmed | FAIL: X failures]
- Open high-priority work: [none affecting deploy | WARNING: N items]

[If FAIL]: Fix the above before deploying. Re-run /deploy when ready.
[If PASS]: Proceeding to strategy selection.
```

---

## Phase 1: Strategy Selection

### 1a. Detect Deployment Tooling

Scan the project root for deployment configuration:

```bash
ls .github/workflows/ Dockerfile docker-compose.yml k8s/ helm/ fly.toml railway.json Procfile .heroku/ 2>/dev/null
```

Build a tooling inventory:

| Signal | Tooling Detected |
|--------|-----------------|
| `.github/workflows/*.yml` with `deploy` or `release` | GitHub Actions |
| `Dockerfile` + `docker-compose.yml` | Docker Compose |
| `k8s/` or `helm/` directory | Kubernetes / Helm |
| `fly.toml` | Fly.io |
| `railway.json` | Railway |
| `Procfile` | Heroku |
| `Makefile` with `deploy` target | Make-based deploy |

Read the detected config files to understand what the deploy pipeline does.

### 1b. Assess Risk Level

Rate the deployment risk based on:

- **Low**: Static site, CLI tool, internal service with no external users
- **Medium**: API with external consumers, feature flag protected, <100 active users
- **High**: Customer-facing service, >100 active users, data migrations, schema changes, no feature flags

Ask the user if the risk level is unclear.

### 1c. Recommend Strategy

Based on tooling and risk:

| Risk | Recommended Strategy | Why |
|------|---------------------|-----|
| Low | **Direct** | Fast, simple. Roll forward if issues appear. |
| Medium | **Canary** | Expose a small slice of traffic first. Monitor, then promote. |
| High | **Blue-Green** | Two identical environments; instant switch back on failure. |
| Any with k8s | **Rolling** | Kubernetes handles this natively; zero-downtime by default. |

Present the recommendation with rationale. Show the alternative strategies briefly.

**Confirm with the user before proceeding:**

> "Recommended strategy: **[strategy]**. Reason: [1 sentence]. Proceed with this strategy, or choose a different one? (direct / canary / blue-green / rolling)"

Wait for confirmation. Proceed with the confirmed strategy.

---

## Phase 2: Execution

Execute the deployment using the confirmed strategy and detected tooling. This phase is project-specific -- adapt to what you found in Phase 1.

### Direct Deployment

The simplest path: deploy the current build directly to the target environment.

**GitHub Actions (push-to-deploy):**

```bash
git push origin main
# Then monitor the workflow run
gh run list --limit 1
gh run watch <run-id>
```

**Docker Compose:**

```bash
docker build -t <image>:<tag> .
docker push <registry>/<image>:<tag>
# On the target host:
docker compose pull && docker compose up -d
```

**Fly.io:**

```bash
fly deploy
```

**Heroku:**

```bash
git push heroku main
```

**Make:**

```bash
make deploy
```

**Manual (no tooling detected):** Guide the user through their specific deploy steps. Ask: "What command do you use to deploy?" Then execute it.

### Canary Deployment

Route a small slice of traffic to the new version before full rollout.

1. Deploy new version alongside existing (e.g., tag as `v2-canary`)
2. Route ~5-10% of traffic to canary (load balancer, feature flag, or header-based routing)
3. Monitor for 10-30 minutes (see Phase 3 monitoring window)
4. If healthy: promote canary to 100%, retire old version
5. If unhealthy: roll back canary (traffic returns to stable) — see Phase 4

**If GitHub Actions or CI handles canary**, trigger the canary workflow and monitor:

```bash
gh workflow run canary-deploy.yml
gh run watch <run-id>
```

### Blue-Green Deployment

Two identical environments: blue (current stable) and green (new release).

1. Deploy new version to the inactive environment (green)
2. Run smoke tests against green before switching
3. Switch the load balancer/DNS to point to green
4. Keep blue running for immediate rollback window (15-60 minutes)
5. If healthy after monitoring window: tear down blue

**If using Kubernetes:**

```bash
kubectl apply -f k8s/green/
kubectl rollout status deployment/<name>-green
# After smoke tests pass:
kubectl patch service/<name> -p '{"spec":{"selector":{"slot":"green"}}}'
```

### Rolling Deployment

Replace instances one by one. Kubernetes handles this natively.

```bash
kubectl set image deployment/<name> <container>=<image>:<tag>
kubectl rollout status deployment/<name>
```

Monitor: `kubectl get pods -w` to watch instance replacement.

### Execution Status

After executing the deploy command, confirm:

```
## Execution: [IN PROGRESS | COMPLETE | FAILED]

- Strategy: [direct | canary | blue-green | rolling]
- Tooling: [GitHub Actions | Docker | k8s | Make | manual]
- Deploy command run: [yes | no]
- Build/image: [tag or commit]
- [If CI]: Workflow URL or run ID for monitoring
```

---

## Phase 3: Monitoring

Post-deployment health verification. Duration varies by strategy.

### Monitoring Window

| Strategy | Minimum Wait | What to Watch |
|----------|-------------|---------------|
| Direct | 5 minutes | Error rate, application logs |
| Canary | 15-30 minutes | Canary error rate vs. baseline |
| Blue-Green | 15 minutes after switch | Error rate on green |
| Rolling | Until all instances replaced | `kubectl rollout status` |

### 3a. Endpoint Health Checks

If the project exposes HTTP endpoints, check them:

```bash
# Basic health endpoint
curl -sf https://<host>/health && echo "OK" || echo "FAIL"

# Or application root
curl -sf https://<host>/ -o /dev/null -w "%{http_code}"
```

If the URL is not known, ask the user: "What URL should I check to confirm the service is up?"

### 3b. Log Review

Scan recent logs for error signals:

**Docker:**
```bash
docker logs <container> --tail=50 --since=5m
```

**Kubernetes:**
```bash
kubectl logs deployment/<name> --tail=50 --since=5m
```

**Fly.io:**
```bash
fly logs --app <app-name>
```

**Local/manual:** Ask the user where logs live and what error patterns to look for.

Look for:
- HTTP 5xx error spikes
- Exception stack traces
- Connection refused / timeout errors
- Out-of-memory signals

### 3c. Health Summary

Report after the monitoring window:

```
## Monitoring: [HEALTHY | DEGRADED | FAILED]

Monitoring window: [N minutes]

- Endpoint check: [pass (HTTP 200) | fail (HTTP NNN) | skipped]
- Log signals: [clean | WARNING: N errors | FAIL: error pattern found]
- Error rate: [baseline | elevated — see rollback criteria]

[If HEALTHY]: Deployment successful. Proceed to close out (see Phase 4).
[If DEGRADED or FAILED]: Rollback criteria triggered — see Phase 4.
```

---

## Phase 4: Rollback Criteria

Define what triggers rollback and how to execute it.

### 4a. Rollback Triggers

**Automatic triggers** (execute rollback immediately, do not wait for manual review):

- Health endpoint returns non-2xx for 3 consecutive checks
- Log scan finds exception rate >5x baseline in the monitoring window
- Process/pod restart loop detected (crash loop)
- `kubectl rollout status` reports `ErrImagePull` or `CrashLoopBackOff`

**Manual triggers** (report to user, ask for decision):

- Error rate elevated but below automatic threshold (1-5x baseline)
- Latency increased significantly but no errors
- User reports confirm something is wrong

### 4b. Rollback Execution

Execute the appropriate rollback based on strategy:

**Direct / Git push:**

```bash
git revert HEAD --no-edit
git push origin main
# Or: redeploy the previous tag
git checkout <previous-tag>
# Then re-run the deploy command
```

**Kubernetes rolling:**

```bash
kubectl rollout undo deployment/<name>
kubectl rollout status deployment/<name>
```

**Blue-Green:**

```bash
# Switch load balancer back to blue
kubectl patch service/<name> -p '{"spec":{"selector":{"slot":"blue"}}}'
# Confirm traffic is back on blue
```

**Canary:**

```bash
# Route 0% traffic to canary
# (tooling-specific — guide the user through their load balancer or feature flag config)
```

**Docker Compose (redeploy previous):**

```bash
docker compose pull <previous-tag> && docker compose up -d
```

### 4c. Post-Rollback

After rollback:

1. Confirm health checks pass on the stable version
2. Create a beads task (or note) for the root cause investigation:

```bash
bd create --title="[service]: investigate deploy failure [date]" --type=task --priority=1 \
  --description="Rollback triggered during deploy. Error signals: [what you found in Phase 3]. Investigate root cause before next deploy attempt."
```

If `.beads/` does not exist, write to `TODO.md`:

```markdown
## Post-rollback: investigate deploy failure [date]
- [ ] Root cause: [what triggered rollback]
- [ ] Fix and verify before next deploy
```

### 4d. Close Out (Successful Deploy)

If no rollback was needed:

```
## Deploy Complete

- Target: [service / environment]
- Strategy: [direct | canary | blue-green | rolling]
- Version: [git tag, commit hash, or image tag]
- Monitoring window: [N minutes, all checks passed]
- Rolled back: no

Next steps:
- Tag the release if not already tagged: `git tag v<version> && git push origin v<version>`
- Close any related beads: `bd close <id>`
- Announce to team if applicable
```

---

## Guidelines

1. **Gate is mandatory.** Never skip Phase 0. A failed readiness gate is not a blocker to skip -- it is a signal to fix before deploying.
2. **Adapt to the project.** This skill detects tooling and adapts. If the project uses a deploy tool not listed, ask the user what command to run and execute it.
3. **Strategy selection needs user confirmation.** Never proceed with a deployment strategy without explicit user confirmation. Deployment is irreversible on short timescales.
4. **Monitoring window is non-negotiable.** Do not declare success before the minimum monitoring window elapses. Stay in Phase 3 until the window passes or a rollback trigger fires.
5. **Rollback first, investigate second.** If rollback triggers fire, execute rollback immediately. Root cause investigation happens after the service is stable.
6. **Be conservative with health signals.** Ambiguous logs or elevated-but-not-alarming error rates should be escalated to the user, not silently accepted.
7. **Compaction resilience.** This skill has 5 phases. Write intermediate state to `memory/scratch/deploy-checkpoint.md` at phase boundaries per `rules/compaction-resilience.md`.
8. **Secrets stay out.** Never write environment variables, credentials, or API keys into commit messages, log output, or any file. Reference them by name only.

## See Also

- `/test-strategy` — verify tests pass before starting Phase 0
- `/tracer` — implement the feature end-to-end before deploying
- `/premortem` — identify deployment failure modes before executing Phase 2
- `/review` — code review before shipping
- `/status` — check current backlog and session state before deploying
