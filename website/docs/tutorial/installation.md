# Installation

There are three ways to get tantex running:

## Option 1: Automated installer (recommended)

The easiest way — download and install in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/tna5/tantex/main/install.sh | sh
```

The script will:
- Detect your OS (Linux, macOS) and CPU architecture (x86_64, ARM64)
- Download the latest binary from GitHub releases
- Offer to install it to `/usr/local/bin` or your current directory

---

## Option 2: Manual download

Download the binary for your system from [GitHub releases](https://github.com/tna5/tantex/releases):

| OS | Architecture | Binary name |
|---|---|---|
| **Linux** | x86_64 | `tantex-linux-x86_64` |
| **macOS** | x86_64 (Intel) | `tantex-macos-x86_64` |
| **macOS** | ARM64 (Apple Silicon) | `tantex-macos-arm64` |

Once downloaded, make it executable and run:

```bash
chmod +x tantex-linux-x86_64  # (use the appropriate name for your system)
./tantex-linux-x86_64
```

To install globally:

```bash
sudo mv tantex-linux-x86_64 /usr/local/bin/tantex
tantex  # now available in PATH
```

---

## Option 3: Build from source

If you want to compile tantex yourself, you'll need:

- **Rust toolchain** — install from [rustup.rs](https://rustup.rs) (requires ≥ 1.75)
- **Bun** (optional) — for building the embedded dashboard; install from [bun.sh](https://bun.sh)

Clone the repository and build:

```bash
git clone https://github.com/tna5/tantex tantex
cd tantex
make
```

The binary is created at `target/release/tantex`.

Or build just the server (skip dashboard rebuild):

```bash
cargo build --release
```

---

## Next steps

Once installed, start the server:

```bash
tantex
```

You should see output like:

```
  ╭─ tantex server starting…
  │ socket  /tmp/tantex.sock
  │ http    :7200
  │
  │ ...
  ╰─ listening on /tmp/tantex.sock and :7200
```

The server opens two interfaces:

- **HTTP** on `http://127.0.0.1:7200` — REST API and embedded dashboard
- **Unix socket** at `/tmp/tantex.sock` — high-throughput binary protocol

See [Getting started](./getting-started.md) for the next steps.
