# Glyph — Windows Build Guide

Save this file to Google Drive before booting into Windows.

---

## Step 1: Install Prerequisites

Open PowerShell as Administrator for all commands below.

### 1a. Install Rust
```powershell
winget install Rustlang.Rustup
```
Then close and reopen PowerShell. Verify:
```powershell
rustc --version
cargo --version
```

### 1b. Install Git
```powershell
winget install Git.Git
```

### 1c. Install Visual Studio Build Tools (required by Rust on Windows)
Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

During install, select:
- **Desktop development with C++** workload
- Ensure "Windows 10/11 SDK" and "MSVC v143 build tools" are checked

This is mandatory. Rust on Windows uses the MSVC linker.

### 1d. Install CMake (required by some Slint dependencies)
```powershell
winget install Kitware.CMake
```
Restart PowerShell after this step.

---

## Step 2: Clone the Repo

```powershell
cd C:\Users\YourName\Documents
git clone https://github.com/YOUR_GITHUB_USERNAME/glyph.git
cd glyph
```

Replace `YOUR_GITHUB_USERNAME` with your actual GitHub username.

---

## Step 3: Build

```powershell
cargo build --release
```

This will take 5-15 minutes on the first run (downloading and compiling all dependencies).

The compiled binary will be at:
```
target\release\glyph.exe
```

---

## Step 4: Test It

```powershell
.\target\release\glyph.exe
```

Glyph should open as a native Windows application.

---

## Step 5: Package for Distribution

### Option A — Simple ZIP (fastest, v1 launch)

1. Create a folder called `Glyph-Windows`
2. Copy `target\release\glyph.exe` into it
3. Zip the folder
4. Upload to GitHub Releases as `Glyph-Windows-x86_64.zip`

### Option B — Installer with NSIS (more polished)

Install NSIS: https://nsis.sourceforge.io/Download

Create a file `glyph_installer.nsi` with:
```nsis
Name "Glyph"
OutFile "Glyph-Setup.exe"
InstallDir "$PROGRAMFILES64\Glyph"
InstallDirRegKey HKLM "Software\Glyph" "Install_Dir"

Section "Install"
  SetOutPath $INSTDIR
  File "target\release\glyph.exe"
  CreateShortcut "$DESKTOP\Glyph.lnk" "$INSTDIR\glyph.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\glyph.exe"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\Glyph.lnk"
  RMDir "$INSTDIR"
SectionEnd
```

Then compile the installer:
```powershell
makensis glyph_installer.nsi
```

This produces `Glyph-Setup.exe` — a proper Windows installer.

---

## Notes

- `settings.json` on Windows will be created in the same directory as the `.exe` (same as Linux behavior)
- Pro license keys work identically — same `GLYPH-PRO-XXXX` format
- No additional DLLs needed — Rust compiles to a single static binary on Windows

---

## Troubleshooting

| Error | Fix |
|---|---|
| `linker 'link.exe' not found` | Visual Studio Build Tools not installed or not in PATH — reinstall them |
| `cmake not found` | Restart PowerShell after installing CMake |
| `error: failed to run custom build command` | Run `cargo clean` then `cargo build --release` again |
| Black window on launch | Update your GPU drivers — Slint uses hardware rendering |
