# Contributing to Domino

Notes for contributors working on the plugin or recorder.

## Repo layout

```
domino/
├── recorder/                # Rust crate: audio capture + local Whisper transcription
├── plugin/                  # Claude Code plugin: slash commands + plugin manifest
│   ├── .claude-plugin/
│   │   └── plugin.json      # plugin manifest
│   └── commands/            # flat .md slash-command prompts
├── .claude-plugin/
│   └── marketplace.json     # marketplace manifest (so /plugin marketplace add resolves)
├── install.sh               # curl-able installer
└── .github/workflows/       # CI and release automation
```

## Plugin conventions

- **Manifest path**: `.claude-plugin/plugin.json` (only the manifest lives inside `.claude-plugin/`; commands and other components sit at the plugin root).
- Only `name` is strictly required in `plugin.json`; `description`, `version`, `author`, `homepage`, `repository`, and `license` are recommended.
- **Slash commands**: flat `.md` files under `commands/`. YAML frontmatter supports `description`, `argument-hint`, `allowed-tools`, `model`, and a few others.

## Local plugin development

Load the in-tree plugin directly from this working copy:

```bash
claude --plugin-dir /path/to/domino/plugin
```

Inside a Claude Code session, reload after edits with `/reload-plugins`.

## Local marketplace testing

To test the marketplace manifest without pushing, `/plugin marketplace add` accepts a local path:

```
/plugin marketplace add /path/to/domino
/plugin install domino@domino
```

Run this from a Claude Code session started **outside** the domino repo so you're simulating a real user install, not loading the plugin as a local dev dependency.

## Building the recorder

Base release build:

```bash
cargo build --release --manifest-path recorder/Cargo.toml
```

If the Swift / ScreenCaptureKit link step fails, retry with the explicit SDK path:

```bash
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX15.4.sdk \
  cargo build --release --manifest-path recorder/Cargo.toml
```

### Running the recorder during development

On machines where the embedded Swift runtime rpath does not resolve cleanly, prefix recorder invocations with:

```bash
export DYLD_FALLBACK_LIBRARY_PATH=/Library/Developer/CommandLineTools/usr/lib/swift-5.5/macosx
```

End users should never need this — the installer verifies Command Line Tools are present, which is what supplies that path. If you hit a `dyld` error in normal use, open an issue with the exact error text and your `xcode-select -p` output.

## Tests

```bash
cargo fmt --manifest-path recorder/Cargo.toml --check
cargo clippy --manifest-path recorder/Cargo.toml -- -D warnings
cargo test --manifest-path recorder/Cargo.toml
```

CI runs these on every push and pull request against `main`.

## Releases

Releases are cut by pushing a `v*` tag. GitHub Actions builds the darwin-arm64 binary, packages it, computes a SHA256, and attaches both to a GitHub Release. See `.github/workflows/release.yml`.

Use `-rc` or `-beta` suffixes for pre-releases so the GitHub Release is marked as pre-release (not surfaced by the "Latest release" widget).

When cutting a release that users should land on via `install.sh`, bump `DEFAULT_VERSION` in `install.sh` to the new tag in the same commit (or immediately after). The installer pins the version on purpose — it never hits GitHub's unauthenticated API for resolution, so it's unaffected by rate limits and CDN staleness at release time.

## Reporting issues

Bugs, feature requests, and questions are welcome as GitHub issues. For suspected security vulnerabilities, see [SECURITY.md](./SECURITY.md).
