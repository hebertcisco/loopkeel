# loopkeel

`loopkeel` is a small Rust orchestration CLI for agentic work cycles: plan, execute, validate, and repeat. It is designed to be called from Claude Code, Codex, or Cursor hooks and tasks. Cursor's native `/loop` remains untouched; this binary is an external companion.

## Install and build

Prebuilt release archives are published for Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64 from [hebertcisco/loopkeel](https://github.com/hebertcisco/loopkeel).

### Linux

Using `curl`:

```sh
curl -fL "https://github.com/hebertcisco/loopkeel/releases/latest/download/loop-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/loop.tar.gz
tar -xzf /tmp/loop.tar.gz -C /tmp
sudo install -m 755 /tmp/loop /usr/local/bin/loop
```

Using `wget`:

```sh
wget -O /tmp/loop.tar.gz "https://github.com/hebertcisco/loopkeel/releases/latest/download/loop-x86_64-unknown-linux-gnu.tar.gz"
tar -xzf /tmp/loop.tar.gz -C /tmp
sudo install -m 755 /tmp/loop /usr/local/bin/loop
```

### macOS

Use `aarch64-apple-darwin` on Apple Silicon and `x86_64-apple-darwin` on Intel Macs.

Using `curl`:

```sh
LOOP_TARGET="aarch64-apple-darwin" # or x86_64-apple-darwin
curl -fL "https://github.com/hebertcisco/loopkeel/releases/latest/download/loop-${LOOP_TARGET}.tar.gz" -o /tmp/loop.tar.gz
tar -xzf /tmp/loop.tar.gz -C /tmp
sudo install -m 755 /tmp/loop /usr/local/bin/loop
```

Using `wget`:

```sh
LOOP_TARGET="aarch64-apple-darwin" # or x86_64-apple-darwin
wget -O /tmp/loop.tar.gz "https://github.com/hebertcisco/loopkeel/releases/latest/download/loop-${LOOP_TARGET}.tar.gz"
tar -xzf /tmp/loop.tar.gz -C /tmp
sudo install -m 755 /tmp/loop /usr/local/bin/loop
```

### Windows

In PowerShell, use `curl.exe` or `wget.exe` so the commands refer to the native download tools rather than PowerShell aliases:

```powershell
$repo = "hebertcisco/loopkeel"
$url = "https://github.com/$repo/releases/latest/download/loop-x86_64-pc-windows-msvc.zip"
curl.exe -fL $url -o "$env:TEMP\loop.zip"
Expand-Archive -Force "$env:TEMP\loop.zip" "$env:TEMP\loop"
New-Item -ItemType Directory -Force "$env:LOCALAPPDATA\Programs\loop" | Out-Null
Copy-Item -Force "$env:TEMP\loop\loop.exe" "$env:LOCALAPPDATA\Programs\loop\loop.exe"
```

The equivalent `wget.exe` download is:

```powershell
wget.exe -O "$env:TEMP\loop.zip" "https://github.com/hebertcisco/loopkeel/releases/latest/download/loop-x86_64-pc-windows-msvc.zip"
```

Add `%LOCALAPPDATA%\Programs\loop` to `PATH` if it is not already available in your shell.

### Cargo

After the crate is published:

```sh
cargo install loopkeel
```

For development from a checkout:

```sh
cargo install --path .
# or: cargo build --release
```

The installed command is `loop` and the CLI also documents `$loop` as an alias. In agent slash-command/task configuration, invoke the external binary as `/loop` or `$loop` according to that tool's convention.

### Homebrew

Once the Homebrew tap is published, install the latest release on macOS or Linux with:

```sh
brew tap hebertcisco/loopkeel
brew install loopkeel
```

Upgrade an existing installation with `brew update && brew upgrade loopkeel`. Remove it with `brew uninstall loopkeel`; remove the tap too with `brew untap hebertcisco/loopkeel` if you no longer need it.

The formula builds `loopkeel` from its versioned release source and installs the `loop` executable.

## Usage

```sh
loop start --goal "Implement and validate the feature" --max-iterations 8
loop start --goal "Fix the bug" --success-command "cargo test" --retries 4
loop step
loop status
loop pause
loop resume
loop stop
```

The goal may also be loaded from a plan file with `--plan path/to/plan.md`. Configuration is read from `.loop.toml`; CLI values override it. Durable files are `.loop/state.json`, `.loop/events.jsonl`, and `.loop/control`. A checkpoint is atomically replaced after every iteration, so `Ctrl+C` leaves a resumable state.

`--agent claude|codex|cursor` selects the adapter. Use `--output json` for structured agent ingestion, or `--output text` for humans. Logging supports `RUST_LOG=info` and `--log-format json`.

## Install the agent skill

The repository includes one portable `SKILL.md` that works with both Claude Code and Codex. Install it into the current project with:

```sh
loop skill install --target both
```

This creates `.claude/skills/loop-orchestrator/SKILL.md` and `.codex/skills/loop-orchestrator/SKILL.md`. To install for the current user instead, use `--scope user`; to replace an existing copy, add `--force`:

```sh
loop skill install --target codex --scope user --force
loop skill path --target claude
```

The skill teaches the host agent how to start, inspect, pause, resume, and validate a loop. It does not replace Claude Code commands or Cursor's native `/loop`; it configures an external companion workflow.

## Agent integration examples

Claude Code or Codex can call `loop start --goal "$TASK" --agent claude` from a hook/task and inspect `loop status` between actions. A completion signal can be sent with `printf complete > .loop/control`; `loop stop` is the intentional stop command.

For Cursor, configure an external task or hook to run `loop start ... --agent cursor`; this does not register or replace Cursor's built-in `/loop`. Cursor can consume `loop status --output json` or the JSON event stream.

## Stop criteria and resilience

Loops stop at `max_iterations`, after `success_command` passes, after an explicit `complete` control command, or on `stop`/interrupt. Transient validation failures use exponential backoff (`backoff_ms * 2^attempt`) up to `retries`; a final failure is reported as a typed error. The async engine uses `tokio::select!` for Ctrl+C, control-file commands, and iteration timers. Events are appended to JSONL and are not retained as an in-memory history.

## Development

```sh
cargo test
cargo run -- status
```

To simulate resume, start a loop, press Ctrl+C, then run `loop status`; the checkpoint remains on disk with the last completed iteration. Start again with the same project directory to continue with a new run. Generated changes are not committed or published automatically.
