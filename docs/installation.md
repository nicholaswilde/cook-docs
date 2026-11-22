# Installation

`cook-docs` is distributed as a prebuilt Rust binary via Homebrew, APT, Scoop, and GitHub Releases, or can be compiled from source with Cargo.

## :beer: Homebrew (macOS / Linux)

Install via [Homebrew](https://brew.sh/) using the [`nicholaswilde/homebrew-tap`](https://github.com/nicholaswilde/homebrew-tap):

```bash
brew install nicholaswilde/tap/cook-docs
```

## :package: APT (Debian / Ubuntu)

Install `.deb` packages (`amd64`, `arm64`, `armhf`, `armel`) from the [`nicholaswilde/apt`](https://github.com/nicholaswilde/apt) repository ([nicholaswilde.io/apt](https://nicholaswilde.io/apt)), which bundles the binary, man page, and shell completions:

### 1. Add Repository Signing Key

```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://nicholaswilde.io/apt/public.key | sudo gpg --dearmor -o /etc/apt/keyrings/nicholaswilde-apt.gpg
sudo chmod a+r /etc/apt/keyrings/nicholaswilde-apt.gpg
```

### 2. Add APT Source

=== "Deb822 (Debian 12+ / Ubuntu 22.04+)"
    ```bash
    sudo tee /etc/apt/sources.list.d/nicholaswilde.sources << 'EOF'
    Types: deb
    URIs: https://nicholaswilde.io/apt
    Suites: stable
    Components: main
    Signed-By: /etc/apt/keyrings/nicholaswilde-apt.gpg
    EOF
    ```

=== "Traditional One-Line (`.list`)"
    ```bash
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/nicholaswilde-apt.gpg] https://nicholaswilde.io/apt stable main" | sudo tee /etc/apt/sources.list.d/nicholaswilde-apt.list
    ```

### 3. Update & Install

```bash
sudo apt update
sudo apt install cook-docs
```

## :icecream: Scoop (Windows)

Install via [Scoop](https://scoop.sh/) using [`nicholaswilde/scoop-bucket`](https://github.com/nicholaswilde/scoop-bucket):

```powershell
scoop bucket add nicholaswilde https://github.com/nicholaswilde/scoop-bucket.git
scoop install nicholaswilde/cook-docs
```

## :inbox_tray: GitHub Releases

Download prebuilt archives (`.tar.gz`, `.zip`) and `.deb` packages for Linux, macOS, and Windows from the [GitHub Releases](https://github.com/nicholaswilde/cook-docs/releases) page.

Once installed, you can upgrade in place with:

```bash
cook-docs self-update
```

## :crab: Build or Install from Source

Requires a [Rust toolchain](https://rustup.rs/) (Rust Edition 2024).

```bash title="Install with Cargo"
cargo install --git https://github.com/nicholaswilde/cook-docs
```

```bash title="Build locally"
git clone https://github.com/nicholaswilde/cook-docs.git
cd cook-docs
cargo build --release
# Binary is written to target/release/cook-docs
```

## :shell: Shell Completions & Man Page

Generate autocompletion scripts for `bash`, `zsh`, `fish`, `powershell`, or `elvish`:

```bash
cook-docs completions bash > ~/.local/share/bash-completion/completions/cook-docs
cook-docs completions zsh > ~/.zfunc/_cook-docs
cook-docs completions fish > ~/.config/fish/completions/cook-docs.fish
```

Generate the UNIX roff man page:

```bash
cook-docs man > cook-docs.1
man ./cook-docs.1
```
