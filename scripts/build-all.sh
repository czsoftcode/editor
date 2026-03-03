#!/usr/bin/env bash
# PolyCredo Editor — sestavení všech distribučních balíčků
# Použití: ./scripts/build-all.sh [--no-upload] [--only=<deb|rpm|flatpak|snap|appimage|exe|freebsd|macos>]

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

UPLOAD=true
ONLY="all"
for arg in "$@"; do
    case "$arg" in
        --no-upload)  UPLOAD=false ;;
        --only=*)     ONLY="${arg#--only=}" ;;
    esac
done

VERSION=$(./scripts/get-version.sh)
LOG_DIR="$ROOT_DIR/target/dist-logs"
CACHE="$HOME/.cache/polycredo-editor"

# LLVM — přidáme všechny nainstalované verze do PATH (clang-cl, lipo, …)
for _LLVM_D in /usr/lib/llvm-*/bin; do [[ -d "$_LLVM_D" ]] && export PATH="$PATH:$_LLVM_D"; done

# ── Barvy ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; DIM='\033[2m'; NC='\033[0m'

SUCCEEDED=(); FAILED=(); SKIPPED=()
MANUAL_STEPS=()
START_TIME=$(date +%s)

# ── Helpery ─────────────────────────────────────────────────────────────────
need()         { command -v "$1" >/dev/null 2>&1; }
elapsed()      { echo $(( $(date +%s) - START_TIME ))s; }
only_matches() { [[ "$ONLY" == "all" || "$ONLY" == "$1" ]]; }

header() {
    echo -e "\n${CYAN}${BOLD}┌──────────────────────────────────────────────────────┐${NC}"
    printf "${CYAN}${BOLD}│  %-52s│${NC}\n" "$1"
    echo -e "${CYAN}${BOLD}└──────────────────────────────────────────────────────┘${NC}"
}

mark_ok() {
    echo -e "  ${GREEN}${BOLD}✔  $1${NC}"
    SUCCEEDED+=("$1")
}

mark_fail() {
    local name="$1" log="$2" hint="${3:-}"
    echo -e "  ${RED}${BOLD}✘  $name${NC}  ${DIM}→ $(basename "$log")${NC}"
    echo -e "${DIM}  Poslední řádky logu:${NC}"
    tail -8 "$log" 2>/dev/null | sed 's/^/    /'
    FAILED+=("$name|$log")
    [[ -n "$hint" ]] && MANUAL_STEPS+=("$name: $hint")
}

mark_skip() {
    echo -e "  ${YELLOW}⊘  $1${NC}  ${DIM}(${2})${NC}"
    SKIPPED+=("$1")
    [[ -n "${3:-}" ]] && MANUAL_STEPS+=("$1: ${3}")
}

upload_to_github() {
    $UPLOAD || return 0
    echo -e "  ${CYAN}↑  Nahrávám na GitHub Releases…${NC}"
    gh release create "v$VERSION" \
        --title "Release v$VERSION" \
        --notes "Sestaveno PolyCredo Editorem" 2>/dev/null || true
    gh release upload "v$VERSION" target/dist/* --clobber 2>/dev/null || true
}

# ════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}${CYAN}PolyCredo Editor — sestavení všech balíčků${NC}"
echo -e "${DIM}Verze: $VERSION  |  Logy: $LOG_DIR${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════${NC}"

# ── Čištění target/dist/ ─────────────────────────────────────────────────
header "Příprava — čištění target/dist/"
if [[ -d target/dist && -n "$(ls -A target/dist/ 2>/dev/null)" ]]; then
    echo -e "  ${DIM}Mažu staré balíčky:${NC}"
    ls target/dist/ | sed 's/^/    /'
    rm -f target/dist/*
    echo -e "  ${GREEN}✔  target/dist/ vyčištěna${NC}"
else
    echo -e "  ${DIM}target/dist/ je prázdná, nic ke smazání${NC}"
fi
mkdir -p target/dist target/debian "$LOG_DIR" \
    "$CACHE/flatpak/build" "$CACHE/flatpak/repo" "$CACHE/flatpak/state"

# ── 1. Debian .deb (release → target/dist/) ─────────────────────────────
header "1/8  Debian — .deb"
if ! only_matches "deb"; then
    :
elif ! need dpkg-deb || ! need dpkg-shlibdeps; then
    mark_skip ".deb" "chybí dpkg-dev" \
        "sudo apt-get install -y dpkg-dev binutils"
elif ! need cargo; then
    mark_skip ".deb" "chybí cargo"
else
    log="$LOG_DIR/deb.log"
    echo -e "  ${DIM}→ packaging/deb/build-deb.sh${NC}"
    if DEB_BUILD_TYPE=deb CARGO_TARGET_DIR="$CACHE/target" ./packaging/deb/build-deb.sh 2>&1 | tee "$log"; then
        mark_ok ".deb → target/dist/"
    else
        mark_fail ".deb" "$log" \
            "zkontroluj $log; nutné: dpkg-dev, binutils, cargo release build"
    fi
fi

# ── 2. Fedora .rpm ───────────────────────────────────────────────────────
header "2/8  Fedora — .rpm"
if ! only_matches "rpm"; then
    :
elif ! need cargo-generate-rpm; then
    mark_skip ".rpm" "chybí cargo-generate-rpm" \
        "cargo install cargo-generate-rpm"
else
    log="$LOG_DIR/rpm.log"
    echo -e "  ${DIM}→ cargo generate-rpm${NC}"
    if ( CARGO_TARGET_DIR="$CACHE/target" cargo generate-rpm -o target/dist/ 2>&1 \
         || (CARGO_TARGET_DIR="$CACHE/target" cargo generate-rpm 2>&1 \
             && mv -f ./*.rpm target/dist/ 2>/dev/null) \
       ) 2>&1 | tee "$log"; then
        mark_ok ".rpm → target/dist/"
    else
        mark_fail ".rpm" "$log" "cargo install cargo-generate-rpm; nutné: rpm-build"
    fi
fi

# ── 3. Flatpak ──────────────────────────────────────────────────────────
header "3/8  Flatpak — .flatpak"
YAML="$ROOT_DIR/.polycredo/sandbox/org.polycredo.Editor.yaml"
if ! only_matches "flatpak"; then
    :
elif ! need flatpak-builder; then
    mark_skip ".flatpak" "chybí flatpak-builder" \
        "sudo apt-get install -y flatpak-builder"
elif [[ ! -f "$YAML" ]]; then
    mark_skip ".flatpak" "chybí org.polycredo.Editor.yaml"
    MANUAL_STEPS+=(".flatpak: vytvoř $YAML")
else
    log="$LOG_DIR/flatpak.log"
    echo -e "  ${DIM}→ flatpak-builder + build-bundle${NC}"
    if ( flatpak-builder --force-clean \
            --state-dir="$CACHE/flatpak/state" \
            --repo="$CACHE/flatpak/repo" \
            "$CACHE/flatpak/build" \
            "$YAML" && \
         flatpak build-bundle "$CACHE/flatpak/repo" \
            "target/dist/polycredo-editor-$VERSION.flatpak" \
            org.polycredo.Editor \
       ) 2>&1 | tee "$log"; then
        mark_ok ".flatpak → target/dist/"
    else
        mark_fail ".flatpak" "$log" \
            "sudo apt-get install -y flatpak-builder; flatpak remote-add --user flathub …"
    fi
fi

# ── 4. Snap ─────────────────────────────────────────────────────────────
header "4/8  Snap — .snap"
export PATH="$PATH:/snap/bin"
if ! only_matches "snap"; then
    :
elif ! need snapcraft; then
    mark_skip ".snap" "chybí snapcraft" \
        "sudo snap install snapcraft --classic"
else
    log="$LOG_DIR/snap.log"
    echo -e "  ${DIM}→ snapcraft pack${NC}"
    if ( sg lxd -c "snapcraft pack --output target/dist/polycredo-editor-$VERSION-amd64.snap" \
            2>/dev/null \
         || snapcraft pack --output "target/dist/polycredo-editor-$VERSION-amd64.snap" \
            2>/dev/null \
         || (snapcraft pack \
             && mv -f ./*.snap "target/dist/polycredo-editor-$VERSION-amd64.snap" 2>/dev/null) \
       ) 2>&1 | tee "$log"; then
        mark_ok ".snap → target/dist/"
    else
        mark_fail ".snap" "$log" \
            "lxd init --auto  (nebo: newgrp lxd po přidání do skupiny)"
    fi
fi

# ── 5. AppImage ──────────────────────────────────────────────────────────
header "5/8  AppImage — .AppImage"
if ! only_matches "appimage"; then
    :
elif ! need cargo-appimage; then
    mark_skip ".AppImage" "chybí cargo-appimage" \
        "cargo install cargo-appimage"
else
    log="$LOG_DIR/appimage.log"
    echo -e "  ${DIM}→ cargo appimage${NC}"
    cp packaging/icons/icon-256.png icon.png 2>/dev/null || true
    if cargo appimage 2>&1 | tee "$log"; then
        APPIMAGE=$(find "$CACHE/target" . -maxdepth 6 \
            -name '*.AppImage' ! -path '*/target/dist/*' 2>/dev/null | head -1)
        if [[ -n "$APPIMAGE" ]]; then
            cp "$APPIMAGE" "target/dist/polycredo-editor-$VERSION-x86_64.AppImage"
            mark_ok ".AppImage → target/dist/"
        else
            mark_fail ".AppImage" "$log" \
                "build OK, ale .AppImage nenalezen — hledej v $CACHE/target/"
        fi
    else
        mark_fail ".AppImage" "$log" \
            "nutné: appimagetool (https://github.com/AppImage/AppImageKit)"
    fi
fi

# ── 6. Windows .exe ──────────────────────────────────────────────────────
header "6/8  Windows — .exe (cross-compile)"
if ! only_matches "exe"; then
    :
elif ! need cargo-xwin; then
    mark_skip ".exe" "chybí cargo-xwin" \
        "cargo install cargo-xwin; rustup target add x86_64-pc-windows-msvc"
else
    log="$LOG_DIR/exe.log"
    echo -e "  ${DIM}→ cargo xwin build --target x86_64-pc-windows-msvc${NC}"
    if cargo xwin build --release \
            --target x86_64-pc-windows-msvc \
            --target-dir "$CACHE/target" 2>&1 | tee "$log"; then
        EXE=$(find \
            "$CACHE/target/x86_64-pc-windows-msvc/release" \
            "$CACHE/target/release" \
            "$CACHE/target" \
            -maxdepth 3 -name 'polycredo-editor.exe' 2>/dev/null | head -1)
        if [[ -n "$EXE" ]]; then
            cp "$EXE" "target/dist/polycredo-editor-$VERSION-x86_64.exe"
            mark_ok ".exe → target/dist/"
        else
            mark_fail ".exe" "$log" \
                "build OK, ale .exe nenalezen — spusť: find $CACHE/target -name '*.exe'"
        fi
    else
        mark_fail ".exe" "$log" \
            "nutné: cargo-xwin, llvm-19, rustup target x86_64-pc-windows-msvc"
    fi
fi

# ── 7. FreeBSD .pkg ──────────────────────────────────────────────────────
header "7/8  FreeBSD — .pkg (cross-compile)"
if ! only_matches "freebsd"; then
    :
else
    missing_bsd=()
    need cross || missing_bsd+=("cross")
    need fpm   || missing_bsd+=("fpm")
    if [[ ${#missing_bsd[@]} -gt 0 ]]; then
        mark_skip ".pkg" "chybí: ${missing_bsd[*]}" \
            "cargo install cross; gem install fpm --no-document; + Docker nebo Podman"
    else
        log="$LOG_DIR/freebsd.log"
        echo -e "  ${DIM}→ cross build --target x86_64-unknown-freebsd + fpm${NC}"
        if ( rustup target add x86_64-unknown-freebsd 2>/dev/null || true && \
             cross build --release --target x86_64-unknown-freebsd --target-dir target && \
             fpm -s dir -t freebsd -n polycredo-editor -v "$VERSION" \
                 --prefix /usr/local \
                 -p "target/dist/polycredo-editor-$VERSION-amd64.pkg" \
                 "target/x86_64-unknown-freebsd/release/polycredo-editor=/bin/polycredo-editor" \
           ) 2>&1 | tee "$log"; then
            mark_ok ".pkg → target/dist/"
        else
            mark_fail ".pkg" "$log" \
                "nutné: cross, fpm, Docker nebo Podman; rustup target x86_64-unknown-freebsd"
        fi
    fi
fi

# ── 8. macOS .dmg / .app ─────────────────────────────────────────────────
header "8/8  macOS — .dmg (cross-compile)"
if ! only_matches "macos"; then
    :
else
    missing_mac=()
    need cargo-zigbuild || missing_mac+=("cargo-zigbuild")
    need zig            || missing_mac+=("zig")

    # Ověření rustup targetů pro macOS
    installed_targets=$(rustup target list --installed 2>/dev/null || true)
    echo "$installed_targets" | grep -q "x86_64-apple-darwin"  || missing_mac+=("rustup-target:x86_64-apple-darwin")
    echo "$installed_targets" | grep -q "aarch64-apple-darwin" || missing_mac+=("rustup-target:aarch64-apple-darwin")

    if [[ ${#missing_mac[@]} -gt 0 ]]; then
        mark_skip "macOS .dmg" "chybí: ${missing_mac[*]}" \
            "Sestavit → macOS → Instalovat macOS závislosti  (nebo: cargo install cargo-zigbuild; curl -L https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz | tar xJ; rustup target add x86_64-apple-darwin aarch64-apple-darwin)"
    else
        log="$LOG_DIR/macos.log"

        # codegen-units=1: redukuje počet object files z ~2000+ na ~50-100 (jeden per crate)
        # → zabraňuje ProcessFdQuotaExceeded při linkování
        # -F: cesta k stub frameworkům (zig 0.15+ nemá bundlované macOS frameworky)
        # -undefined dynamic_lookup: povolí chybějící symboly (řeší je dyld za běhu na macOS)
        MACOS_SDK_STUBS="$HOME/.local/share/macos-sdk-stubs"
        MACOS_RUSTFLAGS="-C codegen-units=1 -C link-arg=-F${MACOS_SDK_STUBS} -C link-arg=-undefined -C link-arg=dynamic_lookup"

        # libobjc.tbd stub: zig 0.15+ neobsahuje libobjc v bundlovaném SDK.
        # Bez tohoto stubu zig linker nedokáže najít -lobjc a linking selže.
        ZB_CACHE="$HOME/.cache/cargo-zigbuild"
        if [ -d "$ZB_CACHE" ]; then
            for _ZB_DIR in "$ZB_CACHE"/*/; do
                _DEPS="$_ZB_DIR/deps"
                if [ -d "$_DEPS" ] && [ ! -f "$_DEPS/libobjc.tbd" ]; then
                    echo -e "  ${DIM}  → vytvářím libobjc.tbd stub v $_DEPS${NC}"
                    cat > "$_DEPS/libobjc.tbd" << 'LIBOBJC_STUB_EOF'
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, x86_64-maccatalyst, arm64-macos, arm64-maccatalyst,
                   arm64e-macos, arm64e-maccatalyst ]
install-name:    '/usr/lib/libobjc.A.dylib'
current-version: 228.0
compatibility-version: 1.0
exports:
  - targets:         [ x86_64-macos, x86_64-maccatalyst, arm64-macos, arm64-maccatalyst,
                       arm64e-macos, arm64e-maccatalyst ]
    symbols:         [ __objc_empty_cache, __objc_empty_vtable, __objc_rootAlloc,
                       __objc_rootAllocWithZone, __objc_rootAutorelease, __objc_rootDealloc,
                       __objc_rootIsDeallocating, __objc_rootRelease, __objc_rootRetain,
                       __objc_rootRetainCount, _class_addIvar, _class_addMethod,
                       _class_addProperty, _class_addProtocol, _class_conformsToProtocol,
                       _class_copyIvarList, _class_copyMethodList, _class_copyPropertyList,
                       _class_copyProtocolList, _class_createInstance, _class_getClassMethod,
                       _class_getImageName, _class_getInstanceMethod, _class_getInstanceSize,
                       _class_getInstanceVariable, _class_getIvarLayout, _class_getMethod,
                       _class_getName, _class_getProperty, _class_getSuperclass,
                       _class_isMetaClass, _class_replaceMethod, _class_respondsToSelector,
                       _class_setIvarLayout, _ivar_getName, _ivar_getOffset,
                       _ivar_getTypeEncoding, _method_copyArgumentType,
                       _method_copyDescription, _method_copyReturnType,
                       _method_getDescription, _method_getImplementation, _method_getName,
                       _method_getNumberOfArguments, _method_getTypeEncoding,
                       _method_setImplementation, _NSObjCBooleanValueProperty,
                       _object_dispose, _object_getClass, _object_getClassName,
                       _object_getIndexedIvars, _object_getInstanceVariable, _object_getIvar,
                       _object_isKindOfClass, _object_setClass, _object_setInstanceVariable,
                       _object_setIvar, _objc_alloc, _objc_allocateClassPair,
                       _objc_allocateProtocol, _objc_alloc_init, _objc_allocWithZone,
                       _objc_autorelease, _objc_autoreleasePoolPop, _objc_autoreleasePoolPush,
                       _objc_autoreleaseReturnValue, _objc_begin_catch,
                       _objc_collectingEnabled, _objc_copyClassList,
                       _objc_copyClassNamesForImage, _objc_copyProtocolList,
                       _objc_copyStruct, _objc_copyWeak, _objc_destroyWeak,
                       _objc_destructInstance, _objc_disposeClassPair, _objc_end_catch,
                       _objc_exception_rethrow, _objc_exception_throw, _objc_fatal,
                       _objc_getAssociatedObject, _objc_getClass, _objc_getClassList,
                       _objc_getMetaClass, _objc_getProperty, _objc_getProtocol,
                       _objc_getRequiredClass, _objc_initWeak, _objc_lookUpClass,
                       _objc_msgSend, _objc_msgSendSuper, _objc_msgSendSuper2,
                       _objc_msgSend_stret, _objc_moveWeak, _objc_registerClassPair,
                       _objc_release, _objc_releaseAndReturn, _objc_retain,
                       _objc_retainAutorelease, _objc_retainAutoreleaseAndReturn,
                       _objc_retainAutoreleaseReturnValue,
                       _objc_retainAutoreleasedReturnValue, _objc_retainedObject,
                       _objc_setAssociatedObject, _objc_setProperty,
                       _objc_setProperty_atomic, _objc_setProperty_atomic_copy,
                       _objc_setProperty_nonatomic, _objc_setProperty_nonatomic_copy,
                       _objc_storeWeak, _objc_storeWeakOrNil, _objc_sync_enter,
                       _objc_sync_exit, _objc_terminate, _objc_unretainedObject,
                       _objc_unretainedPointer, _objc_unsafeClaimAutoreleasedReturnValue,
                       _objc_loadWeak, _objc_loadWeakRetained, _protocol_addMethod,
                       _protocol_addProperty, _protocol_addProtocol,
                       _protocol_conformsToProtocol, _protocol_copyMethodDescriptionList,
                       _protocol_copyPropertyList, _protocol_copyProtocolList,
                       _protocol_isEqual, _protocol_getName, _property_copyAttributeList,
                       _property_copyAttributeValue, _property_getAttributes,
                       _property_getName, _sel_getName, _sel_getUid, _sel_isMapped,
                       _sel_isEqual, _sel_registerName ]
    objc-classes:    [ NSObject ]
    objc-eh-types:   [ NSException ]
LIBOBJC_STUB_EOF
                fi
            done
        fi

        # macOS framework stubs: zig 0.15+ nemá bundlované system frameworky.
        # Vytvoříme minimální TBD soubory → dyld je najde a načte za běhu na macOS.
        if [ ! -d "$MACOS_SDK_STUBS/Foundation.framework" ]; then
            echo -e "  ${DIM}  → vytvářím macOS framework stubs v $MACOS_SDK_STUBS${NC}"
            mkdir -p "$MACOS_SDK_STUBS"
            for _FW_ENTRY in \
                "AppKit:C" "Foundation:C" "CoreFoundation:A" "CoreGraphics:A" \
                "CoreServices:A" "Security:A" "SystemConfiguration:A" \
                "OpenGL:A" "ApplicationServices:A" "CoreVideo:A" "Carbon:A"; do
                _FW="${_FW_ENTRY%%:*}"
                _VER="${_FW_ENTRY##*:}"
                mkdir -p "$MACOS_SDK_STUBS/${_FW}.framework"
                cat > "$MACOS_SDK_STUBS/${_FW}.framework/${_FW}.tbd" << FWEOF
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, x86_64-maccatalyst, arm64-macos, arm64-maccatalyst, arm64e-macos, arm64e-maccatalyst ]
install-name:    '/System/Library/Frameworks/${_FW}.framework/Versions/${_VER}/${_FW}'
current-version: 1.0
compatibility-version: 1.0
FWEOF
            done
        fi

        echo -e "  ${DIM}→ cargo zigbuild — Intel (x86_64) + Apple Silicon (aarch64)${NC}"
        MAC_FAIL=false
        for ARCH_TARGET in x86_64-apple-darwin aarch64-apple-darwin; do
            rustup target add "$ARCH_TARGET" 2>>"$log" || true
            echo -e "  ${DIM}  sestavuji $ARCH_TARGET …${NC}"
            if ! CARGO_TARGET_DIR="$CACHE/target" \
               RUSTFLAGS="$MACOS_RUSTFLAGS" \
               cargo zigbuild --release \
                    --target "$ARCH_TARGET" 2>&1 | tee -a "$log"; then
                MAC_FAIL=true
            fi
        done

        if ! $MAC_FAIL; then
            # ── .app bundle ───────────────────────────────────────────────
            APP_DIR="$CACHE/target/polycredo-editor.app"
            rm -rf "$APP_DIR"
            mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

            cat > "$APP_DIR/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>CFBundleExecutable</key>       <string>polycredo-editor</string>
    <key>CFBundleIdentifier</key>       <string>org.polycredo.editor</string>
    <key>CFBundleName</key>             <string>PolyCredo Editor</string>
    <key>CFBundleDisplayName</key>      <string>PolyCredo Editor</string>
    <key>CFBundlePackageType</key>      <string>APPL</string>
    <key>CFBundleSignature</key>        <string>????</string>
    <key>NSHighResolutionCapable</key>  <true/>
    <key>NSHumanReadableCopyright</key> <string>Copyright © 2024 PolyCredo</string>
</dict></plist>
PLIST
            # Vložíme verzi (PLIST byl heredoc s jednoduchými uvozovkami, proměnná $VERSION se nevyhodnotila)
            sed -i "s|</dict></plist>|    <key>CFBundleVersion</key><string>$VERSION</string>\n    <key>CFBundleShortVersionString</key><string>$VERSION</string>\n</dict></plist>|" \
                "$APP_DIR/Contents/Info.plist"

            BIN_INTEL="$CACHE/target/x86_64-apple-darwin/release/polycredo-editor"
            BIN_ARM="$CACHE/target/aarch64-apple-darwin/release/polycredo-editor"
            BIN_OUT="$APP_DIR/Contents/MacOS/polycredo-editor"

            if need lipo; then
                lipo -create -output "$BIN_OUT" "$BIN_INTEL" "$BIN_ARM" 2>>"$log"
                echo -e "  ${GREEN}✔  Universal Binary (Intel + Apple Silicon)${NC}"
            else
                # Bez lipo — ARM do .app, Intel jako standalone binary
                cp "$BIN_ARM" "$BIN_OUT"
                cp "$BIN_INTEL" \
                    "target/dist/polycredo-editor-$VERSION-x86_64-macos"
                echo -e "  ${YELLOW}⚠  lipo nedostupný — ARM do .app, Intel jako standalone${NC}"
                MANUAL_STEPS+=("macOS: pro Universal Binary nainstaluj: sudo apt-get install -y llvm")
            fi
            chmod +x "$BIN_OUT"

            # Ikona
            [[ -f packaging/icons/icon.icns ]] && \
                cp packaging/icons/icon.icns "$APP_DIR/Contents/Resources/icon.icns"

            # ── DMG / fallback ────────────────────────────────────────────
            TMP_DMG="$CACHE/target/dmg_staging"
            rm -rf "$TMP_DMG" && mkdir -p "$TMP_DMG"
            cp -r "$APP_DIR" "$TMP_DMG/"
            DMG_OUT="target/dist/polycredo-editor-$VERSION-macos.dmg"

            if need create-dmg; then
                echo -e "  ${DIM}→ create-dmg${NC}"
                if create-dmg \
                        --volname "PolyCredo Editor $VERSION" \
                        --window-size 540 380 \
                        --icon-size 128 \
                        --app-drop-link 380 200 \
                        "$DMG_OUT" "$TMP_DMG" 2>&1 | tee -a "$log"; then
                    mark_ok ".dmg → target/dist/"
                else
                    mark_fail "macOS .dmg" "$log" \
                        "create-dmg selhal; zkus: npm install -g create-dmg"
                fi
            elif need genisoimage; then
                echo -e "  ${DIM}→ genisoimage (HFS+)${NC}"
                if genisoimage -V "PolyCredo Editor" -D -R -apple -no-pad \
                        -o "$DMG_OUT" "$TMP_DMG" 2>&1 | tee -a "$log"; then
                    mark_ok ".dmg → target/dist/"
                else
                    # Fallback na .tar.gz
                    tar -czf "target/dist/polycredo-editor-$VERSION-macos.tar.gz" \
                        -C "$(dirname "$APP_DIR")" "$(basename "$APP_DIR")" 2>>"$log"
                    mark_ok ".app.tar.gz → target/dist/"
                    MANUAL_STEPS+=("macOS: konvertuj .tar.gz na .dmg pomocí hdiutil na Macu")
                fi
            else
                # Žádný DMG nástroj — balíme .app jako .tar.gz
                echo -e "  ${YELLOW}⚠  create-dmg ani genisoimage nejsou dostupné — balím jako .tar.gz${NC}"
                if tar -czf "target/dist/polycredo-editor-$VERSION-macos.tar.gz" \
                        -C "$(dirname "$APP_DIR")" "$(basename "$APP_DIR")" 2>>"$log"; then
                    mark_ok ".app.tar.gz → target/dist/"
                else
                    mark_fail "macOS" "$log" "archivace .app selhala"
                fi
                MANUAL_STEPS+=("macOS: pro .dmg nainstaluj: sudo apt-get install -y genisoimage  nebo  npm i -g create-dmg")
            fi
        else
            mark_fail "macOS" "$log" \
                "build selhal i přes dostupné nástroje — zkontroluj $log (možná chybí macOS SDK headers)"
        fi
    fi
fi

# ── GitHub upload ────────────────────────────────────────────────────────
if [[ ${#SUCCEEDED[@]} -gt 0 ]]; then
    header "GitHub — nahrání na Releases"
    upload_to_github
fi

# ════════════════════════════════════════════════════════════════════════════
# Závěrečné shrnutí
# ════════════════════════════════════════════════════════════════════════════
TOTAL=$(( ${#SUCCEEDED[@]} + ${#FAILED[@]} + ${#SKIPPED[@]} ))

echo -e "\n${BOLD}${CYAN}"
echo    "╔══════════════════════════════════════════════════════╗"
printf  "║  Výsledky sestavení  v%-31s║\n" "$VERSION  "
printf  "║  Čas: %-46s║\n" "$(elapsed)  "
echo    "╚══════════════════════════════════════════════════════╝"
echo -e "${NC}"

if [[ ${#SUCCEEDED[@]} -gt 0 ]]; then
    echo -e "${GREEN}${BOLD}Úspěšně sestaveno (${#SUCCEEDED[@]}/${TOTAL}):${NC}"
    for s in "${SUCCEEDED[@]}"; do
        echo -e "  ${GREEN}✔${NC}  $s"
    done
fi

if [[ ${#SKIPPED[@]} -gt 0 ]]; then
    echo -e "\n${YELLOW}${BOLD}Přeskočeno — chybějící nástroje (${#SKIPPED[@]}):${NC}"
    for s in "${SKIPPED[@]}"; do
        echo -e "  ${YELLOW}⊘${NC}  $s"
    done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo -e "\n${RED}${BOLD}Selhalo (${#FAILED[@]}):${NC}"
    for f in "${FAILED[@]}"; do
        name="${f%%|*}"; log="${f##*|}"
        echo -e "  ${RED}✘${NC}  $name  ${DIM}→ $log${NC}"
    done
fi

if [[ ${#MANUAL_STEPS[@]} -gt 0 ]]; then
    echo -e "\n${YELLOW}${BOLD}Vyžaduje ruční zásah:${NC}"
    for m in "${MANUAL_STEPS[@]}"; do
        echo -e "  ${YELLOW}▶${NC}  $m"
    done
fi

echo -e "\n${DIM}Logy: $LOG_DIR/${NC}"
echo -e "${DIM}Balíčky:${NC}"
ls -lh target/dist/ 2>/dev/null | tail -n +2 | sed 's/^/  /'
echo ""

if [[ ${#FAILED[@]} -eq 0 && ${#SKIPPED[@]} -eq 0 ]]; then
    echo -e "${GREEN}${BOLD}Vše sestaveno a nahráno.${NC}\n"
    exit 0
elif [[ ${#FAILED[@]} -gt 0 ]]; then
    exit 1
else
    exit 0
fi
