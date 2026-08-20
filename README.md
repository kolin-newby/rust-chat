# rust-chat

rust-chat is a terminal chat client and server, written in Rust, that speaks two unrelated protocols behind one interface: a custom line-delimited JSON protocol over raw TCP, and the real Matrix protocol via [matrix-sdk](https://github.com/matrix-org/matrix-rust-sdk). It's a learning project — the interesting part isn't the chat itself, it's the `ChatBackend` trait that lets one interactive loop drive either transport interchangeably.

## Features

- **Two interchangeable backends** — a raw-TCP peer-to-peer transport and a Matrix homeserver client, both behind one `ChatBackend` trait (`join_room` / `leave_room` / `send_message` / `poll_events`)
- **Runtime backend selection** — the CLI subcommand picks the backend; `run_interactive` takes `Box<dyn ChatBackend>`, not a concrete type
- **A real Matrix client** — login, initial sync, live event handling, and a background sync task, not a stub
- **A small versioned wire protocol** for the TCP transport (`WireEnvelope`/`WireContent`, JSON, line-delimited)
- **A local Matrix test environment** (`docs/testing/`) — spin up a throwaway homeserver and two accounts with one command, no real Matrix account needed

## Quick Start

> Requires Rust (edition 2021) — install via [rustup](https://rustup.rs)

```bash
cargo build
```

Try the TCP transport with two terminals:

```bash
# terminal 1
cargo run -- server --port 9000 --username alice

# terminal 2
cargo run -- client --host 127.0.0.1 --port 9000 --username bob
```

Type a message and press enter to send it to whichever room you're currently in (starts as `default`). Try `/join myroom`, `/leave`, and `/quit`.

## Usage

```
rust-chat <COMMAND>
```

### `server` — listen for a single incoming TCP connection

```bash
cargo run -- server [--port <PORT>] [--username <NAME>]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--port` | `-p` | `9000` | Port to listen on |
| `--username` | `-u` | `server` | Display name sent with messages |

### `client` — connect to a TCP server

```bash
cargo run -- client --host <HOST> [--port <PORT>] [--username <NAME>]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--host` | `-H` | *(required)* | Server host (IP or hostname) |
| `--port` | `-p` | `9000` | Server port |
| `--username` | `-u` | `client` | Display name sent with messages |

### `matrix` — connect to a Matrix homeserver

```bash
cargo run -- matrix --homeserver <HOMESERVER> --user-id <USER_ID> --password <PASSWORD> [--insecure]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--homeserver` | `-H` | *(required)* | Matrix homeserver name, e.g. `matrix.org` |
| `--user-id` | `-u` | *(required)* | Matrix user ID or localpart to log in as |
| `--password` | `-p` | *(required)* | Account password |
| `--insecure` | | `false` | Connect over plain HTTP and skip `.well-known` discovery — for local test homeservers. See [Testing](#testing) |

`--password` is a plain CLI argument, so it lands in your shell history and is visible via `ps` while running. Don't use a password you care about.

### Interactive commands

Once connected (any backend), the session accepts:

| Command | Effect |
|---|---|
| *(plain text)* | Send as a message to the current room |
| `/join <room>` | Leave the current room (if not `default`) and join `<room>` |
| `/leave` | Leave the current room and return to `default` |
| `/quit` | Exit |

For the `matrix` backend, `<room>` is a Matrix room ID or alias (e.g. `!abc:matrix.org` or `#room:matrix.org`) that the account is already a member of — `rust-chat` doesn't create or discover rooms, only joins/leaves them.

## Testing

```bash
cargo test
```

Unit tests currently cover `protocol/` only (11 tests: `WireEnvelope`/`ChatEvent` conversion, the constructors, `RoomId`, and a JSON round-trip). `p2p.rs` and `matrix.rs` — where the actual I/O and parsing happen — don't have tests yet.

### Exercising the Matrix backend locally

`docs/testing/` spins up a throwaway [Conduit](https://conduit.rs) homeserver in Docker, with no real Matrix account required:

```bash
cd docs/testing
docker compose up -d
./seed.sh
```

`seed.sh` registers two accounts (`acct1`/`acct2`) and a shared room, then prints the exact command to run. It's safe to re-run — it logs in instead of re-registering if the accounts already exist.

```bash
cargo run -- matrix --homeserver localhost:6167 --user-id acct1 --password testpass1 --insecure
```

Tear down with `docker compose down` (add `-v` to also wipe the homeserver's data).

## Architecture

Full C4 model diagrams (Context, Container, Component) live in [`docs/architecture/`](docs/architecture/), covering the backend trait design, both implementations, and the shared protocol module in detail. Start at [`01-context.md`](docs/architecture/01-context.md).

## Project conventions

Commit format, commit chunking, pre-commit review, and documentation-upkeep rules are documented in [`AGENTS.md`](AGENTS.md) — read that before committing changes.

This project is developed with AI coding agent assistance, directed and reviewed by the maintainer rather than generated autonomously.

**Models**

- **Claude Sonnet 5** (Anthropic) — sole AI contributor to date. Co-authored 19 of this repo's 30 commits: the backend architecture (`ChatBackend` trait, `P2PBackend`, `MatrixBackend`), the protocol module and its test suite, the Matrix CLI wiring, the architecture diagrams, this README, and the local Matrix testing setup.

**Skills in use**

Tracked in [skills-lock.json](skills-lock.json): `rust-skills` and `rust-async` (Rust idioms), `conventional-commit` (commit formatting), `c4-model` (architecture diagrams), `good-readme` (this file).

## License

Licensed under the [Apache License 2.0](LICENSE).
