# Stack Research

**Domain:** Rust desktop editor (`eframe/egui`) - milestone `v1.3.1 safe trash delete`
**Researched:** 2026-03-11
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (edition) | 2024 | Implementace move-to-trash, restore a cleanup logiky | Maximalni kontrola nad FS operacemi a error handlingem bez nove runtime vrstvy. |
| Rust `std::fs` + `std::path` | std (toolchain) | Atomicky-ish move (`rename`), fallback copy+remove, validace cest | Pro lokalni projektovy trash je to nejmensi a nejstabilnejsi stack bez dalsich zavislosti. |
| `eframe`/`egui` | `0.31` | UI trigger/delete/restore/cleanup workflow + toasty s chybami | Zapada do existujici architektury single-process multi-window, bez UI rewritu. |
| `serde` + `serde_json` | `1.x` + `1.x` | Volitelny metadata manifest trash polozek (puvodni cesta, timestamp) | Zavislost uz existuje; bezpecnejsi a robustnejsi nez rucni serializace. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `notify` | `7` | Reakce na zmeny v projektu po move/restore operacich | Pouzit jen pro sladeni UI stavu po presunech; nepridavat novy watcher subsystem. |
| `rfd` | `0.15` | Asynchronni file dialog pro restore/cisteni akce | Pouzit pouze pokud akce vyzaduje vyber uzivatele; UI vlakno zustava neblokujici. |
| `tempfile` (dev) | `3` | Izolovane testy pro trash scenare | Pouzit v testech move/restore/collision/cleanup policy. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo check` | Rychla validace kompilace a typu | Povinny minimalni gate po kazdem patchi. |
| `./check.sh` | Projektovy quality gate | Spoustet po logickych zmenach trash workflow. |

## Installation

```bash
# Zadna nova runtime zavislost neni pro v1.3.1 nutna.
# Pouzit existujici stack z Cargo.toml.

cargo check
./check.sh
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `std::fs::rename` + fallback copy/remove | crate `trash` (OS recycle bin) | Jen pokud by produktovy smer vyzadoval OS-integraci misto interniho `.polycredo/trash`. |
| `serde_json` metadata | rucne skladany JSON string | Nikdy pro tento milestone; alternativa nedava vyhodu a zvysuje riziko chyb. |
| Jednoducha cleanup policy (age/count/size) v app logice | DB index (SQLite apod.) | Jen pokud trash naroste do radove tisicu+ polozek a bude potreba pokrocile dotazovani. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Novy async runtime nebo job system | Overengineering mimo scope, zvyseni complexity a riziko regresi | Existujici vlakna/task model aplikace + kratke FS operace mimo UI blokace. |
| Externi DB pro trash metadata | Zbytecna operacni a implementacni rezie pro milestone `v1.3.1` | Jednoduchy JSON manifest (`serde_json`) nebo per-item metadata soubor. |
| Hard delete jako default tok | Neresi bezpecnostni cil milestone (moznost obnovy) | Move-to-trash do `.polycredo/trash` jako primarni cesta. |
| Globalni refactor architektury editoru | Mimo zadany scope capability-focused zmen | Cileny patch jen kolem delete/restore/cleanup bodu. |

## Stack Patterns by Variant

**If soubor zustava ve stejnem filesystemu projektu:**
- Use `std::fs::rename` do `.polycredo/trash/<id>-<name>`.
- Because je to nejrychlejsi a nejmene chybova cesta bez kopirovani dat.

**If `rename` selze kvuli cross-device nebo permission edge case:**
- Use fallback `copy` + `remove_file`/`remove_dir_all` s rollback/error toastem.
- Because zajisti robustni chovani i mimo idealni FS podminky.

**If restore cil uz existuje (collision):**
- Use explicitni conflict policy (rename restored item nebo potvrzeni uzivatelem).
- Because tiche prepisy jsou rizikove a nesmi byt default.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `eframe@0.31` | `egui_extras@0.31` | Drzet stejny minor kvuli API konzistenci UI vrstvy. |
| `serde@1.x` | `serde_json@1.x` | Stabilni serializace trash metadata bez custom parseru. |
| `notify@7` | Rust 2024 crate stack | Dostacujici pro navazani na existujici watcher logiku. |

## Sources

- `.planning/PROJECT.md` - aktivni milestone scope, cile a out-of-scope hranice.
- `Cargo.toml` - aktualni verze stacku a overeni, ze nove crate nejsou nutne.
- Rust std docs (`std::fs`, `std::path`) - validace, ze move/copy/remove scenare lze pokryt bez dalsich knihoven.

---
*Stack research for: PolyCredo Editor v1.3.1 safe trash delete*
*Researched: 2026-03-11*
