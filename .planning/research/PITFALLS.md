# Pitfalls Research

**Domain:** Safe Trash Delete v desktop editoru (Rust/eframe/egui)
**Researched:** 2026-03-11
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Kolize identit pri move do trash

**What goes wrong:**
Dva soubory se stejnym nazvem (nebo opakovane mazani stejne cesty) se v trash prepisi, nebo nelze jednoznacne urcit puvodni lokaci pri restore.

**Why it happens:**
Implementace uklada jen basename bez stabilniho metadata zaznamu (puvodni cesta, cas, duvod kolize).

**How to avoid:**
Pouzit atomicky rename do unikatniho trash id (`timestamp + nonce`) a vedle datoveho souboru ukladat metadata (`original_path`, `deleted_at`, `size`, `hash optional`). Obnovu ridit pres id, ne pres jmeno.

**Warning signs:**
- Restore UI ukazuje vice stejnych nazvu bez puvodni cesty.
- Pri opakovanem delete stejny soubor v trash "zmizi" nebo se prepise.
- Logika restore vybira prvni shodu podle nazvu.

**Phase to address:**
Faze v1.3.1-01 (trash schema + metadata kontrakt) a v1.3.1-02 (move-to-trash implementace).

---

### Pitfall 2: Cross-device move fallback udela hard delete

**What goes wrong:**
Pri rename mezi odlisnymi filesystemy (`EXDEV`) fallback nechtene smaze zdroj driv, nez je cil bezpecne zapsan.

**Why it happens:**
Implementace predpoklada, ze `rename` vzdy funguje atomicky, a fallback copy+delete nema fsync/verify krok.

**How to avoid:**
Implementovat explicitni fallback: `copy -> fsync file -> fsync parent dir -> verify size(optional hash) -> delete source`. Pokud kterykoli krok selze, source nesmazat a vratit chybu do UI toastu.

**Warning signs:**
- Chyby `Invalid cross-device link` v logu bez retry/fallback stopy.
- Uzivatel hlasi ztratu souboru pri mazani z mounted adresare.
- Telemetrie/trace ukazuje delete source i pri failed copy.

**Phase to address:**
Faze v1.3.1-02 (move-to-trash engine) a v1.3.1-06 (error-path testy).

---

### Pitfall 3: Restore prepise existujici soubor bez potvrzeni

**What goes wrong:**
Obnova do puvodni cesty prepise aktualni existujici soubor, uzivatel prijde o novejsi data.

**Why it happens:**
Restore tok neresi konflikt na cilove ceste a chybi UX volba (skip/rename/replace).

**How to avoid:**
Pred restore kontrolovat cil (`exists + is_file/is_dir`) a nabidnout deterministic varianty: `restore as copy (name suffix)`, `cancel`, `replace after confirm`. Vychozi volba musi byt non-destructive.

**Warning signs:**
- Restore probehne bez dialogu i kdyz cil existuje.
- User reporty "po obnove chybi moje novejsi verze".
- Testy pokryvaji restore success, ale ne conflict scenar.

**Phase to address:**
Faze v1.3.1-03 (restore flow + conflict policy) a v1.3.1-05 (UI wiring + i18n texty).

---

### Pitfall 4: Cleanup maze nespravne polozky

**What goes wrong:**
Cleanup podle veku/velikosti smaze i polozky, ktere jeste nemely byt expirovane, nebo smaze metadata bez dat (nekonzistence trash).

**Why it happens:**
Pouziti mtime misto `deleted_at`, neatomicke mazani dvojice `data+meta`, chybejici dry-run/preview.

**How to avoid:**
Opirat TTL o `deleted_at` z metadata, cleanup provadet jako transakcni jednotku nad zaznamem (nejdriv validace vazby data/meta, pak delete oboji). Pro UI nabidnout preview poctu/objemu pred potvrzenim.

**Warning signs:**
- Nahodne "mizeni" cerstve smazanych souboru.
- Trash listing obsahuje sirotci metadata nebo data bez metadata.
- Cleanup kod filtruje pres filesystem times bez metadata.

**Phase to address:**
Faze v1.3.1-04 (cleanup policy + retention) a v1.3.1-06 (integracni testy konzistence).

---

### Pitfall 5: Watcher event storm a stale UI stav

**What goes wrong:**
Move/restore vyvola lavinu filesystem eventu; UI seznam souboru/trash osciluje, duplikuje polozky nebo se neaktualizuje.

**Why it happens:**
Watchery zpracovavaji jednotlive eventy bez deduplikace, nefiltruji interni `.polycredo/trash` zmeny, nebo mixuji interni a user-visible eventy.

**How to avoid:**
Zavedeni event batching (`HashSet<PathBuf>` + debounce tick), explicitni filtrace internich trash operaci pro workspace tree, a oddeleny refresh pipeline pro trash panel.

**Warning signs:**
- Po delete/restore skace vyber v exploreru.
- V logu stovky watcher eventu pro jednu akci.
- Intermitentni flaky testy na file tree refresh.

**Phase to address:**
Faze v1.3.1-05 (watcher/UI integrace) a v1.3.1-06 (stress + race testy).

---

### Pitfall 6: Blokujici I/O v UI vlakne

**What goes wrong:**
Mazani/obnova/cleanup blokuje main thread; UI zamrzne pri vetsich souborech nebo pomalem disku.

**Why it happens:**
Disk operace bez background job queue + bez progress/error signalu z workeru do UI.

**How to avoid:**
Spoustet move/restore/cleanup asynchronne mimo UI vlakno, navratovy stav komunikovat pres kanal (`pending/success/error`) a chyby zobrazovat toastem. UI akce locknout jen lokalne (konkretni item), ne globalne.

**Warning signs:**
- Input lag po stisku Delete/Restore.
- Dlouhe frame times pri disk operacich.
- "Not responding" okna na velkem projektu.

**Phase to address:**
Faze v1.3.1-02 (engine async contract) a v1.3.1-05 (UI integration + progress state).

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Ukladat jen nazev souboru bez metadata | Rychla implementace | Nelze bezpecne restore ani auditovat puvod | Nikdy |
| Ignore error vetve pri cleanup | Mene kodu v prvni iteraci | Tiche ztraty dat, nekonzistentni trash | Nikdy |
| Globalni mutex kolem celeho trash workflow | Snadne "vyresi" race | UI stalls, head-of-line blocking | Jen kratkodobe pro hotfix s ticketem na odstraneni |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| File watcher (`notify`) | Reakce na kazdy event bez batch/debounce | Agregovat eventy do setu a obnovovat UI po ticku |
| Workspace tree UI | Sledovat `.polycredo/trash` stejne jako bezne soubory | Trash drzet jako interni domenu s oddelenym refresh tokem |
| Toast/error pipeline | Chyby move/restore jen logovat | Propagovat chybu uzivateli + navrhnout recovery akci |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full rescan projektu po kazdem delete/restore | Vysoke CPU, lag exploreru | Cileny refresh jen dotcenych cest + debounce | Stovky az tisice souboru |
| Sync cleanup vseho trash najednou | Zamrzani UI | Batch cleanup po blocich s progress signalem | Trash > 1-2 GB nebo HDD |
| Kopirovani velkych souboru v UI threadu | Frame dropy, "not responding" | Worker thread + chunked progress | Jednotky stovek MB |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Restore bez canonicalize/validace cesty | Path traversal mimo projekt | Pred restore overit canonical path a povoleny root projektu |
| Nasledovani symlinku v trash metadatech | Obnova nebo cleanup nad cizimi cestami | Ukladat `symlink` flag a defaultne symlinky ne-followovat |
| Trust filesystem timestampu pro retention | Obchazeni cleanup politiky | Retention ridit pouze pres interni `deleted_at` metadata |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Delete vypada jako hard delete bez zpetne vazby | Neduvera, panika ze ztraty dat | Toast "Presunuto do kosu" + akce "Obnovit" |
| Restore bez konfliktniho dialogu | Nechtene prepsani souboru | Non-destructive default + jasne volby |
| Cleanup bez preview | Uzivatel nevi co zmizi | Preview poctu/velikosti + potvrzeni |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Move-to-trash:** Chybi EXDEV fallback a verify krok - over `rename` fail path testem.
- [ ] **Restore:** Chybi conflict handling na existujicim cili - over testem `restore_when_target_exists`.
- [ ] **Cleanup:** Chybi konzistence `data+meta` mazani - over testem se sirotky.
- [ ] **Watcher integrace:** Chybi deduplikace eventu - over stress testem s burst operacemi.
- [ ] **UI responsiveness:** Chybi async pipeline - over frame-time smoke testem pri velkem souboru.
- [ ] **Error surfacing:** Chyby zustavaji jen v logu - over toast assertions v UI testech.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Kolize/prehazene metadata | HIGH | Stop cleanup, zmrazit nove delete operace, projet trash index verifier, obnovit metadata z journalu nebo fallback na manualni mapping podle inode/size/time |
| Nechtene prepsani pri restore | HIGH | Okamzite ulozit overwritten verzi do nouzoveho backupu (pokud existuje), zablokovat dalsi restore bez conflict policy, pridat guard test a hotfix |
| Event storm/stale UI | MEDIUM | Zapnout watcher debounce fallback, invalidovat cache tree, provest jednorazovy full refresh a nasadit batch dedup patch |
| Zamrzani UI pri trash operacich | MEDIUM | Presunout operace do workeru, docasne omezit cleanup batch size, pridat progress indikator a timeout guard |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Kolize identit pri move do trash | Faze v1.3.1-01/02 | Jednotkove testy uniqueness + metadata roundtrip pro opakovany delete |
| EXDEV fallback udela hard delete | Faze v1.3.1-02 | Integracni test simulujici rename fail a kontrola ze source zustal pri failed copy |
| Restore prepise existujici data | Faze v1.3.1-03/05 | UI+logic test `target exists` s default non-destructive rozhodnutim |
| Cleanup maze nespravne polozky | Faze v1.3.1-04/06 | TTL testy nad `deleted_at`, orphan detection test, dry-run assertion |
| Watcher event storm rozbije UI stav | Faze v1.3.1-05/06 | Stress test burst delete/restore + assert stabilniho tree/trash listu |
| Blokujici I/O v UI vlakne | Faze v1.3.1-02/05 | Performance smoke: delete/restore behem interakce bez frame stall regressi |

## Sources

- `.planning/PROJECT.md` (milestone scope a quality gate kontext)
- Zkusenosti z desktop file manager/editor implementaci (watcher event storms, cross-device move edge cases)
- Bezne failure modes `rename/copy/delete` workflow v POSIX prostredi

---
*Pitfalls research for: v1.3.1 Safe Trash Delete*
*Researched: 2026-03-11*
