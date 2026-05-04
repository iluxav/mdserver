#!/usr/bin/env bash
set -euo pipefail

REPO="iluxav/mdserver"
BIN="mdserver"
INSTALL_DIR="${MDSERVER_INSTALL_DIR:-$HOME/.local/bin}"

err() { echo "error: $*" >&2; exit 1; }

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo linux ;;
        Darwin*) echo darwin ;;
        *) err "unsupported OS: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo x86_64 ;;
        arm64|aarch64) echo aarch64 ;;
        *) err "unsupported arch: $(uname -m)" ;;
    esac
}

target_triple() {
    local os="$1" arch="$2"
    case "$os" in
        linux)  echo "${arch}-unknown-linux-gnu" ;;
        darwin) echo "${arch}-apple-darwin" ;;
    esac
}

latest_tag() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' \
        | head -n1
}

require() {
    command -v "$1" >/dev/null 2>&1 || err "$1 is required but not installed"
}

main() {
    require curl
    require tar
    require uname

    local os arch target version archive url tmp
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="$(target_triple "$os" "$arch")"
    version="${MDSERVER_VERSION:-$(latest_tag)}"
    [ -n "$version" ] || err "could not determine latest release tag for ${REPO}"

    archive="${BIN}-${version}-${target}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${version}/${archive}"

    echo "Downloading ${BIN} ${version} (${target})"
    echo "  ${url}"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    curl -fL --progress-bar "$url" -o "${tmp}/${archive}" \
        || err "download failed: $url"
    tar -xzf "${tmp}/${archive}" -C "$tmp"
    [ -f "${tmp}/${BIN}" ] || err "archive did not contain ${BIN}"

    mkdir -p "$INSTALL_DIR"
    install -m 0755 "${tmp}/${BIN}" "${INSTALL_DIR}/${BIN}"

    echo
    echo "Installed: ${INSTALL_DIR}/${BIN}"
    "${INSTALL_DIR}/${BIN}" --version 2>/dev/null \
        || "${INSTALL_DIR}/${BIN}" --help 2>&1 | head -1 || true

    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo
            echo "Note: ${INSTALL_DIR} is not on your PATH."
            echo "Add it with one of:"
            echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc"
            echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
            ;;
    esac
}

main "$@"
