---
phase: 05-okam-it-aplikov-n-zm-ny-re-imu-sandboxu-po-p-epnut-checkboxu
verified: 2026-03-05T07:30:00Z
status: human_needed
score: 11/11 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 9/11
  gaps_closed:
    - "pending_tab_remap se automaticky vymaže po expiraci SandboxRemapTabs toastu — zombie request není možný (panels.rs:234-245)"
    - "apply_sandbox_mode_change() má dokumentační komentář vysvětlující intentional label-timing chování (state/mod.rs:233-242)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Restart terminálů a label po přepnutí režimu"
    expected: "Starý terminálový proces doběhne, nový terminál se otevře v novém working_dir a label odpovídá novému režimu okamžitě"
    why_human: "Životní cyklus egui_term backendu nelze ověřit staticky; retire_terminal() volá request_graceful_exit() ale faktické pořadí PTY ukončení vyžaduje pozorování za běhu"
  - test: "Tab remap flow — toast interakce"
    expected: "Po přepnutí režimu se zobrazí toast s tlačítky 'Přemapovat taby' / 'Nepřemapovávat'; po kliknutí se taby přemapují, neexistující soubory jsou označeny jako smazané (tab.deleted = true); po ignorování toastu (10 s) pending_tab_remap se automaticky vyčistí"
    why_human: "Vyžaduje otevřené soubory, přepnutí sandbox režimu a pozorování chování tabů v UI včetně ověření cleanup po vypršení"
  - test: "Sync dialog při zapnutí ON"
    expected: "Po zapnutí sandbox ON se zobrazí dialog s plánem synchronizace; po potvrzení se provede sync v background threadu a výsledek je viditelný jako toast"
    why_human: "Závisí na stavu filesystému a spuštěném UI"
---

# Phase 5: Okamžité Aplikování Změn Režimu Sandboxu — Verification Report (Re-verifikace)

**Phase Goal:** Po Save v Settings se sandbox režim aplikuje okamžitě bez reopen, bezpečně pro terminály, file tree, otevřené taby a staged/sync flow
**Verified:** 2026-03-05T07:30:00Z
**Status:** human_needed
**Re-verification:** Ano — po gap-closure plánu 05-04

---

## Re-verifikace: Co se změnilo

Plán 05-04 uzavřel obě SANDBOX-03 mezery nalezené v počáteční verifikaci:

1. **Gap 1 (label-timing):** Uzavřen dokumentací. `apply_sandbox_mode_change()` v `src/app/ui/workspace/state/mod.rs` nyní obsahuje komentář na řádcích 233-242 explicitně dokumentující záměrné okamžité nahrazení labelu a odkaz na verifikaci 05.

2. **Gap 2 (zombie pending_tab_remap):** Uzavřen cleanup logikou. `render_toasts()` v `src/app/ui/panels.rs` obsahuje cleanup blok na řádcích 234-245: ihned po `ws.toasts.retain()` se zkontroluje, zda existuje alespoň jeden toast se `SandboxRemapTabs` akcí; pokud ne, `pending_tab_remap` se nastaví na `None`.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | Změna sandbox režimu se aplikuje až po Save; Cancel vrací původní režim | VERIFIED | `discard_settings_draft()` v settings.rs — Cancel volá `restore_runtime_settings_from_snapshot()` |
| 2 | Po Save se režim nejdříve persistuje na disk, teprve poté se spustí runtime přepnutí | VERIFIED | settings.rs — `draft.try_save()` před nastavením `s.settings = Arc::new(draft)`; při chybě persistu se nenastaví apply |
| 3 | Při změně režimu je změna propagována do všech oken stejného projektu | VERIFIED | `settings_version` AtomicU64 v AppShared (types.rs); mod.rs:557-588 a 744-769 — každý viewport detekuje verzi |
| 4 | OFF režim vyžaduje potvrzení; při otevření jiného dialogu je nabídnuto odložit/udělat hned | VERIFIED | settings.rs — `requires_sandbox_off_confirm()`, `show_sandbox_off_confirm()` modal; workspace/mod.rs:129-143 — toast s SandboxApplyNow/Later |
| 5 | Selhání persistu zobrazí toast s volbou revert / ponechat dočasně | VERIFIED | settings.rs — `sandbox_persist_failure` + dva toasty; workspace/mod.rs:67-110 — `process_sandbox_persist_decision()` |
| 6 | Při změně sandbox režimu se restartují terminály; nové procesy běží v novém rezimu; label-timing je záměrný a zdokumentovaný | VERIFIED | state/mod.rs:217 — `apply_sandbox_mode_change()` s `retire_terminal()` + graceful exit; komentář na řádcích 233-242 dokumentuje intentional label-timing |
| 7 | File tree se přepne na odpovídající root a reload provede | VERIFIED | state/mod.rs:231 — `self.file_tree.load(&target_root)` po změně `file_tree_in_sandbox` |
| 8 | Otevřené taby se mohou přemapovat přes toast; po expiraci toastu se pending_tab_remap automaticky vyčistí | VERIFIED | panels.rs:234-245 — cleanup blok po `ws.toasts.retain()` nulluje `pending_tab_remap` pokud žádný `SandboxRemapTabs` toast neexistuje |
| 9 | OFF je blokováno při staged souborech a uživatel dostane dialog k vyřešení | VERIFIED | settings.rs:525-536 — `should_block_sandbox_off_due_to_staged()` vrací draft na original, `show_sandbox_staged = true` |
| 10 | Sandbox staged bar zůstává viditelná i v OFF, dokud není staged vyřešeno | VERIFIED | workspace/mod.rs:626-655 — `render_sandbox_staged_bar()` je vykreslovana kdykoli `sandbox_staged_files` není prázdné |
| 11 | Při ON se nabídne automaticky sync projektu do sandboxu a při potvrzení se provede | VERIFIED | workspace/mod.rs:191-194 — při `target_mode && !was_enabled` nastaví `ws.sandbox_sync_confirmation = Some(plan)` |

**Score:** 11/11 truths verified

---

### Required Artifacts

| Artifact | Provides | Status | Details |
|----------|----------|--------|---------|
| `src/app/ui/workspace/modal_dialogs/settings.rs` | Save/Cancel flow + potvrzení OFF + toasty + defer | VERIFIED | Implementovány `sandbox_mode_change()`, `requires_sandbox_off_confirm()`, `should_block_sandbox_off_due_to_staged()`, persist-before-apply tok |
| `src/app/ui/workspace/state/mod.rs` | Runtime apply helper + dokumentace label-timing | VERIFIED | `apply_sandbox_mode_change()` na řádku 217; dokumentační komentář na řádcích 233-242 |
| `src/app/mod.rs` | Hook pro detekci změny settings a dispatch apply do oken | VERIFIED | settings_version detekce na řádku 557 a 744; automatické nastavení pending_sandbox_apply |
| `src/app/ui/panels.rs` | Toast rendering + cleanup pending_tab_remap po expiraci | VERIFIED | Cleanup blok na řádcích 234-245; správné zpracování všech toast akcí |
| `src/app/ui/editor/ui.rs` | Přemapování otevřených tabů a zobrazení stavu nenalezených souborů | VERIFIED | `remap_tabs_for_root_change()` v files.rs:228-275; `tab.deleted = !exists` |
| `src/app/ui/workspace/mod.rs` | Blokace OFF + staged bar logika + dialog flow | VERIFIED | `process_pending_sandbox_apply()`, `render_sandbox_staged_bar()`, `process_sandbox_persist_decision()` |
| `src/app/sandbox.rs` | Sync do sandboxu a promítání staged stavu | VERIFIED | `get_sync_plan()`, `sync_plan_to_sandbox()`, `get_staged_files()` implementovány |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| Settings Save | runtime apply | persist -> pending_sandbox_apply | WIRED | settings.rs — try_save() před nastavením settings; pending_sandbox_apply nastaven až po úspěšném persistu |
| settings_version | multi-window | AppShared AtomicU64 + per-viewport detekce | WIRED | mod.rs:557-588 a 744-769 |
| sandbox mode change | terminal restart | apply_sandbox_mode_change() | WIRED | state/mod.rs:233-269 — retire_terminal() + Terminal::new() s novým working_dir |
| sandbox mode change | file tree root | file_tree_in_sandbox + file_tree.load() | WIRED | state/mod.rs:224-231 + panels.rs:60-79 |
| sandbox OFF | staged dialog | should_block_sandbox_off_due_to_staged() | WIRED | settings.rs:525-536 |
| sandbox ON | sync dialog | sandbox_sync_confirmation = Some(plan) | WIRED | workspace/mod.rs:191-194 |
| render_toasts (retain) | pending_tab_remap cleanup | cleanup po expiraci SandboxRemapTabs toastu | WIRED | panels.rs:234-245 — nový cleanup blok z plánu 05-04 |
| toast action SandboxRemapTabs | editor tab remap | pending_tab_remap.take() -> remap_tabs_for_root_change() | WIRED | panels.rs:323-333 |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| SANDBOX-01 | 05-01 | Změna sandbox režimu se po Save aplikuje okamžitě bez reopen a propaguje se do všech oken stejného projektu | SATISFIED | settings_version mechanismus + pending_sandbox_apply ve všech viewportech |
| SANDBOX-02 | 05-01 | Save/Cancel semantika zachovaná; OFF vyžaduje potvrzení; persist před runtime apply; při chybě persistu volba revert/keep | SATISFIED | Kompletní tok v settings.rs + workspace/mod.rs |
| SANDBOX-03 | 05-02 + 05-04 | Runtime apply restartuje terminály, přepíná file tree root a přemapovává otevřené taby | SATISFIED | Terminály a file tree: state/mod.rs; remap: opt-in toast (panels.rs:323-333); cleanup zombie requestu: panels.rs:234-245; label-timing zdokumentován: state/mod.rs:233-242 |
| SANDBOX-04 | 05-03 | OFF je blokováno při staged souborech; ON nabízí automatický sync do sandboxu | SATISFIED | settings.rs + sandbox.rs + workspace/mod.rs |

Všechny čtyři požadavky jsou v `REQUIREMENTS.md` označeny jako `[x]` (splněno).

### Anti-Patterns Found

Žádné nové anti-patterny. Původně nalezená `pending_tab_remap` zombie situace je opravena cleanup logikou v panels.rs.

### Human Verification Required

#### 1. Restart terminálů a label timing

**Test:** Spusť dlouho běžící proces v terminálu (např. `sleep 30`). Přepni sandbox režim a ulož (Save). Sleduj zda starý process doběhl před tím než se label změní.
**Expected:** Label okamžitě odpovídá novému režimu (záměrné chování zdokumentované v kódu); starý proces doběhl gracefully (nebyl kill -9) v `retired_terminals`.
**Why human:** Životní cyklus procesu v egui_term nelze ověřit staticky; `retire_terminal()` volá `request_graceful_exit()` ale pořadí PTY ukončení vyžaduje pozorování za běhu.

#### 2. Tab remap flow — toast interakce a cleanup

**Test:** Otevři 3-4 soubory z projektu. Zapni sandbox režim a ulož. Pockej 10+ sekund bez kliknutí na toast "Přemapovat taby". Po vypršení toastu přepni ještě jednou a sleduj stav tabů.
**Expected:** Toast vyprší a pending_tab_remap se automaticky vymaže (žádná zombie žádost). Taby zůstávají na původních cestách. Při kliknutí "Přemapovat taby" se taby přemapují; soubory neexistující v novém rootu jsou označeny jako smazané (červeně).
**Why human:** Vyžaduje otevřené soubory, přepnutí sandbox režimu a pozorování chování tabů a cleanup v UI.

#### 3. Sync dialog při zapnutí ON

**Test:** Uprav soubory v projektu (ne v sandboxu). Zapni sandbox ON a ulož. Zkontroluj že se zobrazí dialog s plánem synchronizace. Potvrď sync a zkontroluj výsledek.
**Expected:** Dialog zobrazí plán (počty souborů); po potvrzení se soubory zkopírují do sandboxu; toast potvrzení.
**Why human:** Závisí na stavu filesystému a spuštěném UI.

---

### Gaps Summary

Po gap-closure plánu 05-04 nezbývají žádné automaticky verifikovatelné mezery. Všech 11 truths je VERIFIED. SANDBOX-03 je nyní plně splněn:

- Zombie `pending_tab_remap` je nemožný — cleanup v `render_toasts()` zajistí nullování ihned po expiraci příslušného toastu.
- Label-timing je zdokumentovaný záměr — komentář v `apply_sandbox_mode_change()` vysvětluje proč se label mění okamžitě a odkazuje na tuto verifikaci.

Zbývají pouze 3 položky pro lidské ověření (chování za běhu), které nelze ověřit staticky.

---

_Verified: 2026-03-05T07:30:00Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification: Yes — after gap closure plan 05-04_
