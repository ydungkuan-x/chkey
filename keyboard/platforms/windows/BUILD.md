# Windows Builds

The Windows package supports only these 64-bit targets:

- `x86_64-pc-windows-msvc`: Windows x86_64 64-bit
- `aarch64-pc-windows-msvc`: Windows ARM64 64-bit

Install the Rust targets once from the workspace root:

```powershell
rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc
```

Build debug binaries:

```powershell
cargo build-x64
cargo build-arm64
```

Build release binaries:

```powershell
cargo build-x64-release
cargo build-arm64-release
```

Check without linking an executable:

```powershell
cargo check-x64
cargo check-arm64
```

The binaries are written to `target/x86_64-pc-windows-msvc/` and
`target/aarch64-pc-windows-msvc/` respectively.

Both targets require the matching Visual Studio C++ build tools. The ARM64
build also requires the ARM64 MSVC build tools from Visual Studio.