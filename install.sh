#!/usr/bin/env bash
set -euo pipefail

REPO="nitinm21/domino"
BIN_NAME="domino-recorder"
INSTALL_DIR_PRIMARY="/usr/local/bin"
INSTALL_DIR_FALLBACK="${HOME}/.local/bin"

# Pinned release. Bumped as part of the release cut (see CONTRIBUTING.md).
# Override at call time with: DOMINO_VERSION=vX.Y.Z curl ... | DOMINO_VERSION=vX.Y.Z sh
DEFAULT_VERSION="v0.1.0-rc4"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
err() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

# 1. Platform check
[[ "$(uname -s)" == "Darwin" ]] || err "Domino currently supports macOS only."
[[ "$(uname -m)" == "arm64" ]] || err "Domino v0.1.0 ships an arm64 binary only. Intel Mac users: build from source (see README)."

# 2. Xcode Command Line Tools check
if ! xcode-select -p >/dev/null 2>&1; then
  err "Xcode Command Line Tools are required. Install with: xcode-select --install"
fi

# 3. Determine install dir
if [[ -w "${INSTALL_DIR_PRIMARY}" ]]; then
  INSTALL_DIR="${INSTALL_DIR_PRIMARY}"
  SUDO=""
elif sudo -n true 2>/dev/null; then
  INSTALL_DIR="${INSTALL_DIR_PRIMARY}"
  SUDO="sudo"
else
  INSTALL_DIR="${INSTALL_DIR_FALLBACK}"
  SUDO=""
  mkdir -p "${INSTALL_DIR}"
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) log "Note: ${INSTALL_DIR} is not on your PATH. Add it to your shell profile:"
       log "    export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
  esac
fi

# 4. Resolve release version
# DOMINO_VERSION override lets us pin a specific tag (e.g. a pre-release candidate
# during acid-testing). Default path hits /releases/latest which returns the most
# recent non-prerelease, non-draft release.
VERSION="${DOMINO_VERSION:-${DEFAULT_VERSION}}"
log "Installing ${BIN_NAME} ${VERSION}"

ASSET="${BIN_NAME}-${VERSION}-darwin-arm64.tar.gz"
SHA_ASSET="${ASSET}.sha256"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

# 5. Download
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
log "Downloading ${ASSET}..."
curl -fsSL -o "${TMP}/${ASSET}" "${BASE_URL}/${ASSET}"
curl -fsSL -o "${TMP}/${SHA_ASSET}" "${BASE_URL}/${SHA_ASSET}"

# 6. Verify SHA256
log "Verifying SHA256..."
(cd "${TMP}" && shasum -a 256 -c "${SHA_ASSET}" >/dev/null) || err "SHA256 verification failed. Aborting."

# 7. Extract
log "Extracting..."
tar -xzf "${TMP}/${ASSET}" -C "${TMP}"
[[ -x "${TMP}/${BIN_NAME}" ]] || err "Extracted archive did not contain an executable ${BIN_NAME}."

# 8. Strip quarantine attribute (Gatekeeper)
xattr -d com.apple.quarantine "${TMP}/${BIN_NAME}" 2>/dev/null || true

# 9. Install
log "Installing to ${INSTALL_DIR}/${BIN_NAME}..."
${SUDO} install -m 0755 "${TMP}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

# 10. Post-install smoke test
log "Verifying install..."
"${INSTALL_DIR}/${BIN_NAME}" --help >/dev/null 2>&1 || err "Installed binary failed to run. See README troubleshooting."

# 11. Next step
cat <<EOF

$(printf '\033[1;32m')Installed ${BIN_NAME} ${VERSION} to ${INSTALL_DIR}/${BIN_NAME}$(printf '\033[0m')

Next step: inside Claude Code, run

    /plugin marketplace add ${REPO}
    /plugin install domino@domino

Then record a meeting:

    /mstart
    ... hold the meeting ...
    /mstop

First-run notes:
  - macOS will prompt for Microphone and Screen Recording permissions on first use.
  - The Whisper model (~466 MB) downloads once to ~/.domino/models/ on first /mstop.
EOF
