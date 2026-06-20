.PHONY: all build dashboard server clean dev

TARGET ?= native
CARGO_TARGET_FLAGS :=

ifeq ($(TARGET),linux)
	CARGO_TARGET_FLAGS = --target x86_64-unknown-linux-gnu
endif

all: build

build: dashboard server

dashboard:
	cd dashboard && bun install && bun run generate

server:
	cargo build --release $(CARGO_TARGET_FLAGS)

clean:
	cargo clean
	rm -rf dashboard/.nuxt dashboard/.output

dev:
	@echo "→ Start tantex first:  cargo run"
	cd dashboard && bun run dev
