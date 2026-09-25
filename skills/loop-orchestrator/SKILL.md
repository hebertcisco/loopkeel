---
name: loop-orchestrator
description: Run and supervise resilient agentic work loops with loopkeel. Use when a task benefits from repeated plan, execute, validate, pause, resume, or checkpointed work.
---

# Loop orchestrator

Use the external `loop` binary to coordinate a bounded work cycle. Keep the user's goal as the loop goal and let the agent perform the actual implementation work.

## Start a loop

```sh
loop start --goal "<task goal>" --agent claude
```

Use `--agent codex` in Codex and `--agent cursor` in Cursor. For repository validation, add a command such as `--success-command "cargo test"` and configure `--retries` and `--timeout` as needed.

## Observe and control

```sh
loop status --output json
loop pause
loop resume
loop stop
```

Use `loop step` when the user wants manual iteration control. A running loop also accepts `pause`, `resume`, `stop`, and `complete` through `.loop/control`. The loop writes checkpoints to `.loop/state.json` and append-only iteration events to `.loop/events.jsonl`.

Do not claim success only because an iteration ran. Inspect `loop status` and the configured validation command. Resume an interrupted loop from its checkpoint instead of discarding the existing `.loop` state.
