.PHONY: all fmt build app check test

all: build

test:
	cargo nextest run
	cargo nextest run -p wezterm-escape-parser # no_std by default

check:
	cargo check
	cargo check -p wezterm-escape-parser
	cargo check -p wezterm-cell
	cargo check -p wezterm-surface
	cargo check -p wezterm-ssh

app:
	PROFILE=debug ./scripts/build.sh --app-only

build:
	cargo build $(BUILD_OPTS) -p kaku -p kaku-gui -p wezterm-mux-server-impl

fmt:
	cargo +nightly fmt
