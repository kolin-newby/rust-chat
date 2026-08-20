# Level 1 — System Context — rust-chat

> **Diagram type**: System Context
> **Scope**: rust-chat as seen from the outside — who runs it and which external systems it talks to.
> **Audience**: Everyone (technical and non-technical); suitable for anyone orienting to the project for the first time.

## Overview

rust-chat is a terminal chat client/server written in Rust. A user runs it from the command line in exactly one of three modes per invocation: as a TCP server waiting for a single incoming connection, as a TCP client connecting out to a peer, or as a Matrix client authenticating against a Matrix homeserver. It holds no durable state of its own and has no UI beyond the terminal — every session is ephemeral and entirely driven by whichever external system it's talking to for that run.

This diagram answers *"what is rust-chat, and what does it interact with?"* before any decomposition into containers or components.

## Diagram

```mermaid
C4Context
    title System Context diagram for rust-chat

    Person(user, "User", "Runs rust-chat from a terminal to send and receive chat messages.")
    System(rust_chat, "rust-chat", "Terminal chat client/server. Speaks a custom line-delimited JSON protocol over raw TCP, or the Matrix protocol - never both in the same run.")
    System_Ext(tcp_peer, "TCP Peer", "Another rust-chat instance (or any compatible line-delimited JSON speaker) reachable over a raw TCP socket.")
    System_Ext(matrix_homeserver, "Matrix Homeserver", "A Matrix protocol homeserver (e.g. matrix.org) that authenticates the user and owns room state and history.")

    Rel(user, rust_chat, "Issues commands (server/client/matrix, /join, /leave, /quit) to and reads chat output from", "CLI / terminal")
    Rel(rust_chat, tcp_peer, "Exchanges line-delimited JSON chat envelopes with", "raw TCP")
    Rel(rust_chat, matrix_homeserver, "Authenticates, syncs room state with, and sends/receives messages via", "Matrix Client-Server API / HTTPS")
```

## Legend

- **Person / actor**: human user of the system
- **System (in scope)**: rust-chat — the subject of this diagram
- **External system**: out-of-scope system rust-chat interacts with (TCP Peer, Matrix Homeserver)
- No custom colors, shapes, or border styles used — Mermaid C4 default rendering

## Elements

| Element | Type | Technology | Responsibility |
|---|---|---|---|
| User | Person | — | Runs rust-chat from a terminal; the only human actor. |
| rust-chat | System (in scope) | Rust, Tokio | Terminal chat client/server. Picks one of two independent transports per invocation (raw TCP or Matrix) — detailed at Container/Component level. |
| TCP Peer | System_Ext | — (any line-delimited JSON speaker) | The other end of a direct TCP chat session. The protocol is symmetric, so a "peer" is typically just another rust-chat instance. |
| Matrix Homeserver | System_Ext | Matrix Client-Server API | Owns rooms, membership, and message history. rust-chat is a thin client against it. |

## Key relationships

| From | To | Intent | Protocol / Technology |
|---|---|---|---|
| User | rust-chat | Issues commands to and reads chat output from | CLI / terminal |
| rust-chat | TCP Peer | Exchanges line-delimited JSON chat envelopes with | raw TCP |
| rust-chat | Matrix Homeserver | Authenticates, syncs room state with, and sends/receives messages via | Matrix Client-Server API / HTTPS |

## Notable architectural decisions

- **Two independent transports, chosen at invocation, not at runtime.** Which external system rust-chat talks to is fixed by the CLI subcommand (`server`/`client`/`matrix`) for the entire process lifetime — a single run never talks to both a TCP peer and a Matrix homeserver. This is the project's central design choice, and it's already visible at Context level.
- **No system of record inside rust-chat.** For the TCP transport, the relationship is genuinely peer-to-peer — either side could be "the client." For Matrix, the homeserver is the actual system of record for room state and history; rust-chat caches nothing durably.

## Assumptions

- "TCP Peer" is drawn as a single external system for simplicity. In practice a `server` run accepts exactly one incoming connection and a `client` run connects to exactly one address — there is no multi-peer fan-out today. This is confirmed from the code (`P2PBackend::listen`/`connect`), not inferred.
- Matrix Homeserver's specific identity (which server) is supplied by the user at the CLI (`--homeserver`); this diagram represents the class of external system, not a specific instance.

## Links to other levels

- ↓ [Level 2 — Container diagram](./02-container.md) — zoom inside rust-chat (spoiler: it's one container)
