# Contributing to Kaku

## Setup

```bash
# Clone the repository
git clone https://github.com/tw93/Kaku.git
cd Kaku
```

## Development

```bash
# Verify code
make check

# Run tests
make test

# Quick local build → dist/Kaku.app (debug, fastest)
make app
```

## Build Release

```bash
# Build application and DMG (release, native)
./scripts/build.sh
# Outputs: dist/Kaku.app and dist/Kaku.dmg
```

## Pull Requests

1. Fork and create a branch from `main`
2. Make changes
3. Run checks: `make check`
4. Commit and push
5. Open PR targeting `main`

CI will verify formatting, linting, and tests.
