# Install the Boundra CLI

## Native Release

Download the archive for your platform from the matching GitHub Release, verify
it against `checksums-sha256.txt`, and place `boundra` on your `PATH`.

Supported preview archives:

- Linux x64: `x86_64-unknown-linux-gnu`
- macOS Intel: `x86_64-apple-darwin`
- macOS Apple Silicon: `aarch64-apple-darwin`
- Windows x64: `x86_64-pc-windows-msvc`

Verify installation:

```bash
boundra --version
boundra --help
```

## Source Fallback

With a stable Rust toolchain:

```bash
cargo install --git https://github.com/qtaghdi/boundra --package boundra-cli
```

The native release is preferred because it does not require a Rust toolchain.
