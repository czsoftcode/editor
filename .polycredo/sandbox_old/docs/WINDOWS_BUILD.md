# Compiling for Windows (x86_64-pc-windows-msvc)

This document describes how to cross-compile PolyCredo Editor for Windows from a Linux environment (Debian/Ubuntu) using the `cargo-xwin` tool.

## Prerequisites

To successfully cross-compile, you must have the following components installed in your system:

### 1. Rust Target
Install the standard library for the target platform:
```bash
rustup target add x86_64-pc-windows-msvc
```

### 2. LLVM Toolchain
Windows MSVC builds require `clang`, `lld`, and `llvm-lib` (a linker and library manager compatible with MSVC):
```bash
sudo apt-get update
sudo apt-get install -y clang lld
```

### 3. Cargo-XWin
This tool automatically downloads the necessary Windows SDK and CRT (C Runtime) libraries directly from Microsoft without requiring a Windows license or Visual Studio installation:
```bash
cargo install cargo-xwin
```

## Compilation Procedure

The build is performed using the `cargo xwin` command. It is important to ensure that LLVM tools are in your `PATH` (in some distributions, they may be located in `/usr/lib/llvm-XX/bin`).

### Standard Release Build:
```bash
# Ensure the path to LLVM binaries is in PATH (adjust version if necessary)
export PATH=$PATH:/usr/lib/llvm-19/bin 

# Run the build (cargo will automatically use the optimal number of jobs)
cargo xwin build --release --target x86_64-pc-windows-msvc
```

The resulting binary will be located at:
`target/x86_64-pc-windows-msvc/release/polycredo-editor.exe`

## Things to Watch Out For (Architectural Specifics)

### 1. Inter-Process Communication (IPC)
Windows does not support Unix Domain Sockets (`std::os::unix::net`). The implementation in `src/ipc.rs` has been adapted:
- On **Unix**, it continues to use `.sock` files.
- On **Windows**, it uses **local TCP sockets** on `127.0.0.1`.
- The randomly assigned port number is stored in an `ipc.port` file in the application's config directory. Clients read this file to determine where to connect to the primary instance.

### 2. File Paths
When working with paths (`PathBuf`) in code compiled for Windows, keep in mind:
- Using `canonicalize()` on Windows may return UNC paths (starting with `\?`), which some external tools might not support.
- Always use `Path` methods for path separators instead of hardcoded `/` or ``.

### 3. Dependencies (ring, zstd-sys)
Some crates (e.g., `ring` or `zstd-sys`) compile native C/C++ code. Thanks to `cargo-xwin` and the installed `clang-cl` (part of the `clang` package), these libraries compile automatically and link correctly against the Windows CRT.

### 4. System Warnings
When adding new UI elements or functionalities, use `#[cfg(target_os = "windows")]` for Windows-specific code and remember to handle unused variables/imports on other platforms using `_` or `#[cfg]`.
