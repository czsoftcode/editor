# Debian balíček (`.deb`)

Tato složka obsahuje vše potřebné pro vytvoření instalačního balíčku pro Debian/Ubuntu.
Balíček instaluje aplikaci zobrazovanou jako `PolyCredo Editor`.

## Co balíček instaluje

- binárku: `/usr/bin/polycredo-editor`
- desktop entry: `/usr/share/applications/polycredo-editor.desktop`
- ikonu: `/usr/share/icons/hicolor/scalable/apps/polycredo-editor.svg`
- postinst/postrm hooky pro refresh menu a icon cache

Kategorie aplikace v menu je `Development`, takže se zobrazí v sekci „Vývoj“.

## Build balíčku

```bash
./packaging/deb/build-deb.sh
```

Výstup:

```text
target/debian/polycredo-editor_<verze>_<arch>.deb
```

Volitelné proměnné:

- `DEB_VERSION` — explicitní verze balíčku (jinak se počítá z `Cargo.toml` + `.build_number`)
- `DEB_ARCH` — cílová architektura (jinak `dpkg --print-architecture`)
- `DEB_MAINTAINER` — maintainer pole v control souboru

Příklad:

```bash
DEB_VERSION=0.2.42 DEB_MAINTAINER="CZ Softcode <dev@example.com>" ./packaging/deb/build-deb.sh
```

## Instalace

```bash
sudo apt install ./target/debian/polycredo-editor_<verze>_<arch>.deb
```
