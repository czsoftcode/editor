#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKG_NAME="polycredo-editor"
APP_NAME="PolyCredo Editor"
BIN_SOURCE="$ROOT_DIR/target/release/polycredo-editor"
PKG_BUILD_ROOT="$ROOT_DIR/target/deb-build"
STAGE_DIR="$PKG_BUILD_ROOT/$PKG_NAME"
DEBIAN_DIR="$STAGE_DIR/DEBIAN"
ASSETS_DIR="$ROOT_DIR/packaging/deb"
OUTPUT_DIR="${1:-$ROOT_DIR/target/debian}"
ARCH="${DEB_ARCH:-$(dpkg --print-architecture)}"

default_maintainer() {
    local git_name git_mail
    git_name="$(git -C "$ROOT_DIR" config user.name || true)"
    git_mail="$(git -C "$ROOT_DIR" config user.email || true)"
    if [[ -n "$git_name" && -n "$git_mail" ]]; then
        printf '%s <%s>\n' "$git_name" "$git_mail"
    else
        printf '%s\n' "PolyCredo Editor Team <noreply@local>"
    fi
}

MAINTAINER="${DEB_MAINTAINER:-$(default_maintainer)}"

compute_version() {
    local cargo_version major minor build_number
    cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n1)"
    if [[ -z "$cargo_version" ]]; then
        echo "Nepodařilo se načíst verzi z Cargo.toml" >&2
        exit 1
    fi

    IFS='.' read -r major minor _ <<<"$cargo_version"
    build_number="$(tr -d '[:space:]' < "$ROOT_DIR/.build_number")"
    if [[ -z "$build_number" ]]; then
        build_number="0"
    fi
    printf '%s.%s.%s\n' "$major" "$minor" "$build_number"
}

normalize_dep_list() {
    awk -F',' '
        {
            for (i = 1; i <= NF; i++) {
                dep = $i
                gsub(/^[[:space:]]+/, "", dep)
                gsub(/[[:space:]]+$/, "", dep)
                if (dep != "" && !seen[dep]++) {
                    if (out != "") {
                        out = out ", " dep
                    } else {
                        out = dep
                    }
                }
            }
        }
        END {
            if (out == "") {
                print "libc6, libgcc-s1"
            } else {
                print out
            }
        }
    '
}

compute_depends() {
    local shlibs_output shlibs_depends manual_depends
    shlibs_output="$(dpkg_shlibdeps_wrapper "$STAGE_DIR/usr/bin/polycredo-editor")"
    shlibs_depends="$(printf '%s\n' "$shlibs_output" | sed -n 's/^shlibs:Depends=//p')"

    # Rust GUI stack část knihoven načítá dynamicky přes dlopen, takže přidáme
    # minimální runtime balíčky pro X11/Wayland/OpenGL a fonty s Unicode podporou.
    manual_depends="libx11-6, libxcb1, libxkbcommon0, libwayland-client0, libwayland-cursor0, libwayland-egl1, libegl1, libgl1, fonts-dejavu-core, fonts-noto-core, fonts-noto-ui-core, fonts-noto-mono, fonts-symbola, fonts-noto-color-emoji"
    printf '%s\n' "${shlibs_depends}, ${manual_depends}" | normalize_dep_list
}

dpkg_shlibdeps_wrapper() {
    local binary="$1"
    local tmp
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' RETURN
    mkdir -p "$tmp/debian"
    cat > "$tmp/debian/control" <<EOF
Source: $PKG_NAME
Section: devel
Priority: optional
Maintainer: $MAINTAINER
Standards-Version: 4.6.2

Package: $PKG_NAME
Architecture: any
Depends: \${shlibs:Depends}, \${misc:Depends}
Description: $APP_NAME
 Rust desktop editor.
EOF
    (
        cd "$tmp"
        dpkg-shlibdeps -O "$binary" 2>/dev/null || true
    )
}

build_binary() {
    echo "==> Building release binary"
    cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"
    if [[ ! -x "$BIN_SOURCE" ]]; then
        echo "Release binary nebyla nalezena: $BIN_SOURCE" >&2
        exit 1
    fi
}

prepare_stage() {
    echo "==> Preparing staging tree"
    rm -rf "$STAGE_DIR"
    mkdir -p \
        "$DEBIAN_DIR" \
        "$STAGE_DIR/usr/bin" \
        "$STAGE_DIR/usr/share/applications" \
        "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps" \
        "$OUTPUT_DIR"

    install -m 0755 "$BIN_SOURCE" "$STAGE_DIR/usr/bin/polycredo-editor"
    install -m 0644 "$ASSETS_DIR/polycredo-editor.desktop" "$STAGE_DIR/usr/share/applications/polycredo-editor.desktop"
    install -m 0644 "$ASSETS_DIR/polycredo-editor.svg" "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps/polycredo-editor.svg"
    install -m 0755 "$ASSETS_DIR/postinst" "$DEBIAN_DIR/postinst"
    install -m 0755 "$ASSETS_DIR/postrm" "$DEBIAN_DIR/postrm"
}

write_control() {
    local version depends
    version="${DEB_VERSION:-$(compute_version)}"
    depends="$(compute_depends)"

    cat > "$DEBIAN_DIR/control" <<EOF
Package: $PKG_NAME
Version: $version
Section: devel
Priority: optional
Architecture: $ARCH
Maintainer: $MAINTAINER
Depends: $depends
Description: $APP_NAME
 Jednoduchý textový editor napsaný v Rustu s eframe/egui.
 .
 Obsahuje AI polyglot code editor se stromem souborů, více panely a integrovaným terminálem.
EOF
}

build_deb() {
    local version deb_path
    version="$(sed -n 's/^Version: //p' "$DEBIAN_DIR/control")"
    deb_path="$OUTPUT_DIR/${PKG_NAME}_${version}_${ARCH}.deb"
    echo "==> Building package: $deb_path"
    dpkg-deb --build --root-owner-group "$STAGE_DIR" "$deb_path" >/dev/null
    echo "==> Done: $deb_path"
}

main() {
    build_binary
    prepare_stage
    write_control
    build_deb
}

main "$@"
