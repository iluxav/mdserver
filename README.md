# mdserver

A markdown-native HTTP resource server. Point it at a directory of `.md` files and get:

- Every file served at its path (`/docs/intro.md` → raw markdown)
- Every directory served as an auto-generated markdown index (`/docs` → table of files & subdirectories with titles and section headings)
- A `/introspect` endpoint returning the full tree as JSON with merkle-style hashes
- Lazy in-memory caching: stat-based per-file metadata + per-directory rendered index, so repeat reads avoid disk I/O

## Install

### One-liner (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/iluxav/mdserver/main/install.sh | bash
```

The script detects your OS and CPU architecture, downloads the matching binary from the latest GitHub Release, and installs it to `~/.local/bin/mdserver`.

Supported targets:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

### Pin a specific version

```bash
curl -fsSL https://raw.githubusercontent.com/iluxav/mdserver/main/install.sh | MDSERVER_VERSION=v0.1.0 bash
```

### Custom install directory

```bash
curl -fsSL https://raw.githubusercontent.com/iluxav/mdserver/main/install.sh | MDSERVER_INSTALL_DIR=/usr/local/bin bash
```

### macOS Gatekeeper

If macOS blocks the binary on first run with "cannot be opened because the developer cannot be verified", clear the quarantine flag:

```bash
xattr -d com.apple.quarantine ~/.local/bin/mdserver
```

### Make sure it's on your PATH

If `~/.local/bin` isn't already on your PATH, add it:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc   # or ~/.zshrc
```

### Manual download

Grab a tarball directly from the [releases page](https://github.com/iluxav/mdserver/releases), extract `mdserver`, and put it anywhere on your PATH.

## Run

```bash
mdserver --root /path/to/your/markdown --bind 127.0.0.1:8080
```

| Flag         | Env var           | Default          | Notes                                |
|--------------|-------------------|------------------|--------------------------------------|
| `--root`     | `MDSERVER_ROOT`   | required         | Directory to serve                   |
| `--bind`     | `MDSERVER_BIND`   | `127.0.0.1:8080` | Listen address (host:port)           |

Open `http://127.0.0.1:8080/` in a browser. Hit `/introspect` for the JSON tree.

## Chrome extension (optional)

The `ext/` folder in this repo is a Chrome MV3 extension that:

- Renders any `text/markdown` HTTP response as styled HTML in dark mode
- On `.md` file URLs, adds an Edit toggle that opens [MDX Editor](https://mdxeditor.dev/) in-page
- Shows a left sidebar tree from `/introspect` with the current path highlighted
- Saves edits back to the server via HTTP `PUT`

To install:

1. Clone this repo
2. Open `chrome://extensions`
3. Enable **Developer mode**
4. Click **Load unpacked** and pick the `ext/` folder

## Build from source

Requires Rust **1.85+**.

```bash
git clone https://github.com/iluxav/mdserver.git
cd mdserver
cargo build --release
./target/release/mdserver --root ./docs
```

To rebuild the Chrome extension's MDX Editor bundle (only needed if you change `ext/editor-src/`):

```bash
cd ext/editor-src
npm install
npm run build
```

## License

MIT
