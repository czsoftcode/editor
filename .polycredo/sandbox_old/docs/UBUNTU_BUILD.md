# Building Snap Package for Ubuntu (PolyCredo Editor)

This guide describes how to locally build the `.snap` package of PolyCredo Editor on an Ubuntu system (or any other distribution supporting Snapd and LXD/Multipass).

## 1. Prerequisites

Before the first build, it is necessary to have the required tools installed:

```bash
sudo snap install snapcraft --classic
sudo snap install lxd
# Initialize LXD (if not already done)
sudo lxd init --auto
# Add user to the lxd group (requires re-login or 'newgrp lxd')
sudo usermod -a -G lxd $USER
```

## 2. Environment Preparation (Important!)

If you have **Docker** installed, its default network configuration blocks internet access inside LXD build containers. Before running `snapcraft`, perform:

```bash
# Temporarily enable network traffic forwarding
sudo iptables -P FORWARD ACCEPT
```

*Note: This setting will revert to the original state after a system reboot.*

## 3. Building the Package

The build takes place in an isolated environment (container) as defined in `snap/snapcraft.yaml`.

```bash
# Clean up previous attempts
snapcraft clean

# Build the package (the result will be in target/dist/)
mkdir -p target/dist
snapcraft pack --output target/dist/polycredo-editor.snap
```

## 4. Installation and Testing

After a successful build, you can install the package locally using the `--dangerous` flag (since it is not signed by the Snap Store):

```bash
sudo snap install target/dist/polycredo-editor.snap --classic --dangerous
```

## 5. Size Optimization (Target Folder)

The project includes optimizations in `Cargo.toml` to limit the size of debug information and build artifacts:

- `profile.dev.debug = 1`: Reduces space requirements in `target/debug/deps/` by tens of GBs.
- `profile.release.strip = true`: Automatically shrinks the final binary by removing symbols.

To free up space after building, you can use:
```bash
cargo clean
```

## 6. Troubleshooting

### Error: "A network related operation failed"
- Check if Docker is running: `systemctl is-active docker`. If yes, apply step 2 (`iptables -P FORWARD ACCEPT`).
- Verify IP forwarding on the host: `sudo sysctl -w net.ipv4.ip_forward=1`.

### Building in "Destructive Mode"
If LXD still fails due to network issues, you can build the package directly on the host system. **This is not recommended** as it installs build dependencies directly into your OS:
```bash
snapcraft pack --destructive-mode
```
