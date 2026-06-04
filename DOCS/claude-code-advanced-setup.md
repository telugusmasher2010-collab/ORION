# Advanced Setup

System requirements, platform-specific installation, version management, and uninstallation for Claude Code.

This page covers system requirements, platform-specific installation details, updates, and uninstallation. For a guided walkthrough of your first session, see the quickstart. If you've never used a terminal before, see the terminal guide.

## System requirements

Claude Code runs on the following platforms and configurations:

* **Operating system**:
  * macOS 13.0+
  * Windows 10 1809+ or Windows Server 2019+
  * Ubuntu 20.04+
  * Debian 10+
  * Alpine Linux 3.19+
* **Hardware**: 4 GB+ RAM, x64 or ARM64 processor
* **Network**: internet connection required
* **Shell**: Bash, Zsh, PowerShell, or CMD. Native Windows setups require Git for Windows. WSL setups do not.
* **Location**: Anthropic supported countries

### Additional dependencies

* **ripgrep**: usually included with Claude Code. If search fails, check troubleshooting documentation.

## Install Claude Code

### Install Methods

**macOS, Linux, WSL:**
```bash
curl -fsSL https://claude.ai/install.sh | bash
```

**Windows PowerShell:**
```powershell
irm https://claude.ai/install.ps1 | iex
```

**Windows CMD:**
```batch
curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd && del install.cmd
```

If you see `The token '&&' is not a valid statement separator`, you're in PowerShell, not CMD. If you see `'irm' is not recognized as an internal or external command`, you're in CMD, not PowerShell. Your prompt shows `PS C:\` when you're in PowerShell and `C:\` without the `PS` when you're in CMD.

**Native Windows setups require Git for Windows.** Install it first if you don't have it. WSL setups do not need it.

Native installations automatically update in the background to keep you on the latest version.

### Homebrew Installation

```bash
brew install --cask claude-code
```

Homebrew offers two casks. `claude-code` tracks the stable release channel, which is typically about a week behind and skips releases with major regressions. `claude-code@latest` tracks the latest channel and receives new versions as soon as they ship.

Homebrew installations do not auto-update. Run `brew upgrade claude-code` or `brew upgrade claude-code@latest`, depending on which cask you installed, to get the latest features and security fixes.

### WinGet Installation

```powershell
winget install Anthropic.ClaudeCode
```

WinGet installations do not auto-update. Run `winget upgrade Anthropic.ClaudeCode` periodically to get the latest features and security fixes.

### After Installation

After installation completes, open a terminal in the project you want to work in and start Claude Code:

```bash
claude
```

If you encounter any issues during installation, see the troubleshooting guide.

## Set up on Windows

You can run Claude Code natively on Windows or inside WSL. Pick based on where your projects are located and which features you need:

| Option         | Requires                      | Sandboxing | When to use                     |
| -------------- | ----------------------------- | ---------- | ------------------------------- |
| Native Windows | Git for Windows               | Not supported | Windows-native projects       |
| WSL 2          | WSL 2 enabled                 | Supported  | Linux toolchains or sandboxed  |
| WSL 1          | WSL 1 enabled                 | Not supported | If WSL 2 is unavailable       |

### Option 1: Native Windows with Git Bash

Install Git for Windows, then run the install command from PowerShell or CMD. You do not need to run as Administrator.

Whether you install from PowerShell or CMD only affects which install command you run. Your prompt shows `PS C:\Users\YourName>` in PowerShell and `C:\Users\YourName>` without the `PS` in CMD.

After installation, launch `claude` from PowerShell, CMD, or Git Bash. Claude Code uses Git Bash internally to execute commands regardless of where you launched it. If Claude Code can't find your Git Bash installation, set the path in your settings.json file:

```json
{
  "env": {
    "CLAUDE_CODE_GIT_BASH_PATH": "C:\\Program Files\\Git\\bin\\bash.exe"
  }
}
```

Claude Code can also run PowerShell natively on Windows. The PowerShell tool is rolling out progressively; set `CLAUDE_CODE_USE_POWERSHELL_TOOL=1` to opt in or `0` to opt out.

### Option 2: WSL

Open your WSL distribution and run the Linux installer from the install instructions above. You install and launch `claude` inside the WSL terminal, not from PowerShell or CMD.

## Alpine Linux and musl-based distributions

The native installer on Alpine and other musl/uClibc-based distributions requires `libgcc`, `libstdc++`, and `ripgrep`. Install these using your distribution's package manager, then set `USE_BUILTIN_RIPGREP=0`.

This example installs the required packages on Alpine:

```bash
apk add libgcc libstdc++ ripgrep
```

Then set `USE_BUILTIN_RIPGREP` to `0` in your settings.json file:

```json
{
  "env": {
    "USE_BUILTIN_RIPGREP": "0"
  }
}
```

## Verify your installation

After installing, confirm Claude Code is working:

```bash
claude --version
```

For a more detailed check of your installation and configuration, run:

```bash
claude doctor
```

## Authenticate

Claude Code requires a Pro, Max, Team, Enterprise, or Console account. The free Claude.ai plan does not include Claude Code access. You can also use Claude Code with a third-party API provider like Amazon Bedrock, Google Vertex AI, or Microsoft Foundry.

After installing, log in by running `claude` and following the browser prompts.

## Update Claude Code

Native installations automatically update in the background. You can configure the release channel to control whether you receive updates immediately or on a delayed stable schedule, or disable auto-updates entirely. Homebrew, WinGet, and Linux package manager installations require manual updates.

### Auto-updates

Claude Code checks for updates on startup and periodically while running. Updates download and install in the background, then take effect the next time you start Claude Code.

Homebrew, WinGet, apt, dnf, and apk installations do not auto-update. For Homebrew, run `brew upgrade claude-code` or `brew upgrade claude-code@latest`, depending on which cask you installed. For WinGet, run `winget upgrade Anthropic.ClaudeCode`. For Linux package managers, see the upgrade commands in Advanced installation options.

Homebrew keeps old versions on disk after upgrades. Run `brew cleanup` periodically to reclaim disk space.

### Configure release channel

Control which release channel Claude Code follows for auto-updates with the `autoUpdatesChannel` setting:

* `"latest"`, the default: receive new features as soon as they're released
* `"stable"`: use a version that is typically about one week old, skipping releases with major regressions

Configure this via `/config` → **Auto-update channel**, or add it to your settings.json file:

```json
{
  "autoUpdatesChannel": "stable"
}
```

### Pin a minimum version

The `minimumVersion` setting establishes a floor. Background auto-updates and `claude update` refuse to install any version below this value.

Add it to your settings.json file:

```json
{
  "autoUpdatesChannel": "stable",
  "minimumVersion": "2.1.100"
}
```

### Disable auto-updates

Set `DISABLE_AUTOUPDATER` to `"1"` in the `env` key of your settings.json file:

```json
{
  "env": {
    "DISABLE_AUTOUPDATER": "1"
  }
}
```

`DISABLE_AUTOUPDATER` only stops the background check; `claude update` and `claude install` still work. To block all update paths, including manual updates, set `DISABLE_UPDATES` instead.

### Update manually

To apply an update immediately without waiting for the next background check, run:

```bash
claude update
```

## Advanced installation options

### Install a specific version

The native installer accepts either a specific version number or a release channel (`latest` or `stable`).

To install the latest version:

**macOS, Linux, WSL:**
```bash
curl -fsSL https://claude.ai/install.sh | bash
```

**Windows PowerShell:**
```powershell
irm https://claude.ai/install.ps1 | iex
```

**Windows CMD:**
```batch
curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd && del install.cmd
```

To install the stable version:

**macOS, Linux, WSL:**
```bash
curl -fsSL https://claude.ai/install.sh | bash -s stable
```

**Windows PowerShell:**
```powershell
& ([scriptblock]::Create((irm https://claude.ai/install.ps1))) stable
```

**Windows CMD:**
```batch
curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd stable && del install.cmd
```

To install a specific version number:

**macOS, Linux, WSL:**
```bash
curl -fsSL https://claude.ai/install.sh | bash -s 2.1.89
```

**Windows PowerShell:**
```powershell
& ([scriptblock]::Create((irm https://claude.ai/install.ps1))) 2.1.89
```

**Windows CMD:**
```batch
curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd 2.1.89 && del install.cmd
```

### Install with Linux package managers

Claude Code publishes signed apt, dnf, and apk repositories. Replace `stable` with `latest` for the rolling channel.

#### apt (Debian and Ubuntu)

```bash
sudo install -d -m 0755 /etc/apt/keyrings
sudo curl -fsSL https://downloads.claude.ai/keys/claude-code.asc \
  -o /etc/apt/keyrings/claude-code.asc
echo "deb [signed-by=/etc/apt/keyrings/claude-code.asc] https://downloads.claude.ai/claude-code/apt/stable stable main" \
  | sudo tee /etc/apt/sources.list.d/claude-code.list
sudo apt update
sudo apt install claude-code
```

Verify the GPG key fingerprint: `gpg --show-keys /etc/apt/keyrings/claude-code.asc` should report `31DD DE24 DDFA B679 F42D 7BD2 BAA9 29FF 1A7E CACE`.

To upgrade later: `sudo apt update && sudo apt upgrade claude-code`

#### dnf (Fedora and RHEL)

```bash
sudo tee /etc/yum.repos.d/claude-code.repo <<'EOF'
[claude-code]
name=Claude Code
baseurl=https://downloads.claude.ai/claude-code/rpm/stable
enabled=1
gpgcheck=1
gpgkey=https://downloads.claude.ai/keys/claude-code.asc
EOF
sudo dnf install claude-code
```

To upgrade later: `sudo dnf upgrade claude-code`

#### apk (Alpine Linux)

```sh
wget -O /etc/apk/keys/claude-code.rsa.pub \
  https://downloads.claude.ai/keys/claude-code.rsa.pub
echo "https://downloads.claude.ai/claude-code/apk/stable" >> /etc/apk/repositories
apk add claude-code
```

Verify the downloaded key: `sha256sum /etc/apk/keys/claude-code.rsa.pub` should report `395759c1f7449ef4cdef305a42e820f3c766d6090d142634ebdb049f113168b6`.

To upgrade later: `apk update && apk upgrade claude-code`

### Install with npm

You can also install Claude Code as a global npm package. The package requires Node.js 18 or later.

```bash
npm install -g @anthropic-ai/claude-code
```

The npm package installs the same native binary as the standalone installer. Supported npm install platforms are `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`, `linux-x64-musl`, `linux-arm64-musl`, `win32-x64`, and `win32-arm64`.

Do NOT use `sudo npm install -g` as this can lead to permission issues and security risks.

### Binary integrity and code signing

Each release publishes a `manifest.json` containing SHA256 checksums for every platform binary. The manifest is signed with an Anthropic GPG key.

#### Verify the manifest signature

**Step 1: Download and import the public key**
```bash
curl -fsSL https://downloads.claude.ai/keys/claude-code.asc | gpg --import
```

Display the fingerprint of the imported key:
```bash
gpg --fingerprint security@anthropic.com
```

Confirm the output includes: `31DD DE24 DDFA B679 F42D  7BD2 BAA9 29FF 1A7E CACE`

**Step 2: Download the manifest and signature**
```bash
REPO=https://downloads.claude.ai/claude-code-releases
VERSION=2.1.89
curl -fsSLO "$REPO/$VERSION/manifest.json"
curl -fsSLO "$REPO/$VERSION/manifest.json.sig"
```

**Step 3: Verify the signature**
```bash
gpg --verify manifest.json.sig manifest.json
```

A valid result reports `Good signature from "Anthropic Claude Code Release Signing <security@anthropic.com>"`.

**Step 4: Check the binary against the manifest**

Compare the SHA256 checksum of your downloaded binary with the value listed in `manifest.json`.

Linux:
```bash
sha256sum claude
```

macOS:
```bash
shasum -a 256 claude
```

Windows PowerShell:
```powershell
(Get-FileHash claude.exe -Algorithm SHA256).Hash.ToLower()
```

#### Platform code signatures

* **macOS**: signed by "Anthropic PBC" and notarized by Apple. Verify with `codesign --verify --verbose ./claude`.
* **Windows**: signed by "Anthropic, PBC". Verify with `Get-AuthenticodeSignature .\claude.exe`.
* **Linux**: binaries are not individually code-signed. Verify integrity with the manifest signature.

## Uninstall Claude Code

### Native installation

**macOS, Linux, WSL:**
```bash
rm -f ~/.local/bin/claude
rm -rf ~/.local/share/claude
```

**Windows PowerShell:**
```powershell
Remove-Item -Path "$env:USERPROFILE\.local\bin\claude.exe" -Force
Remove-Item -Path "$env:USERPROFILE\.local\share\claude" -Recurse -Force
```

### Homebrew installation

If you installed the stable cask:
```bash
brew uninstall --cask claude-code
```

If you installed the latest cask:
```bash
brew uninstall --cask claude-code@latest
```

### WinGet installation

```powershell
winget uninstall Anthropic.ClaudeCode
```

### apt / dnf / apk

#### apt
```bash
sudo apt remove claude-code
sudo rm /etc/apt/sources.list.d/claude-code.list /etc/apt/keyrings/claude-code.asc
```

#### dnf
```bash
sudo dnf remove claude-code
sudo rm /etc/yum.repos.d/claude-code.repo
```

#### apk
```sh
apk del claude-code
sed -i '\|downloads.claude.ai/claude-code/apk|d' /etc/apk/repositories
rm /etc/apk/keys/claude-code.rsa.pub
```

### npm

```bash
npm uninstall -g @anthropic-ai/claude-code
```

### Remove configuration files

Remove Claude Code settings and cached data:

**macOS, Linux, WSL:**
```bash
rm -rf ~/.claude
rm ~/.claude.json
rm -rf .claude
rm -f .mcp.json
```

**Windows PowerShell:**
```powershell
Remove-Item -Path "$env:USERPROFILE\.claude" -Recurse -Force
Remove-Item -Path "$env:USERPROFILE\.claude.json" -Force
Remove-Item -Path ".claude" -Recurse -Force
Remove-Item -Path ".mcp.json" -Force
```

Note: Removing configuration files will delete all your settings, allowed tools, MCP server configurations, and session history.
