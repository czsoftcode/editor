#!/usr/bin/env bash
# PolyCredo Editor — sestavení všech distribučních balíčků
# Použití: ./scripts/build-all.sh [--no-upload] [--only=<deb|rpm|flatpak|snap|appimage|exe|freebsd>]

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
header "1/7  Debian — .deb"
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
header "2/7  Fedora — .rpm"
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
header "3/7  Flatpak — .flatpak"
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
header "4/7  Snap — .snap"
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
header "5/7  AppImage — .AppImage"
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
header "6/7  Windows — .exe (cross-compile)"
export PATH="$PATH:/usr/lib/llvm-19/bin"
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
header "7/7  FreeBSD — .pkg (cross-compile)"
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
