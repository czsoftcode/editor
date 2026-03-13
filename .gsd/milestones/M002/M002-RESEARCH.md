# M002: Local History — Research

**Date:** 2026-03-13

## Summary

Backend `LocalHistory` v `src/app/local_history.rs` (184 řádků) je funkční se třemi operacemi: `take_snapshot`, `get_history`, `cleanup`. Používá xxhash pro deduplikaci obsahu a timestamp-based filename konvenci (`{ts}_{hash}.txt`). **Hlavní problém: nikdo nikdy neodešle `FsChangeResult::LocalHistory` do kanálu** — receiver `background_io_rx` existuje ve `WorkspaceState`, ale sender (`background_io_tx`) není nikde v kódu. Kanál je vždy `None`. Celý pipeline producent→background handler je mrtvý kód.

UI pro historii neexistuje vůbec — adresář `src/app/ui/workspace/history/` je prázdný. Tab bar nemá context menu (pravý klik na tab nic nedělá), context menu existuje jen na editor body (copy/paste). Diff view (`diff_view.rs`) je modal pro AI diff s Accept/Reject flow — jeho `similar`-based diff rendering logiku (barvy, LayoutJob) lze přímo znovupoužít, ale UI kontejner (modal s footer actions) není vhodný pro inline history browsing.

**Doporučení:** Postupovat od backendu ven. Nejprve propojit snapshot producenta na save hook (nejnižší riziko, ověřitelné testem), pak přidat `get_snapshot_content()` metodu, a nakonec stavět UI — split view adaptovaný z markdown split patternu s diff zvýrazněním znovupoužitým z `diff_view.rs`. Tab context menu je nový koncept v kódu — bude potřeba přidat `context_menu()` na tab selectable label v `render/tabs.rs`.

## Recommendation

Riskiest-first: split view v editoru je největší neznámá (editor dosud nemá koncept dual-pane mode), proto ho ověřit co nejdříve po propojení backendu. Markdown split pattern (`render/markdown.rs`) je přesný vzor — dvě ScrollArea se split handle, resizable ratio. Adaptovat tento pattern pro dva read-only panely s diff zvýrazněním.

Save hook patří do `files.rs` v metodě `save()` — po úspěšném `fs::write` odeslat content přes kanál do background handleru. Kanál `background_io_rx` už existuje, stačí vytvořit sender a propojit ho při inicializaci workspace.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Text diffing | `similar` crate (2.7.0, už v Cargo.toml) | Stabilní, používaný v `diff_view.rs`. Žádná nová závislost. |
| Content hashing/deduplikace | `xxhash_rust::xxh3` (už v local_history.rs) | Rychlý, bez kolizí pro tento use case. |
| Split panel layout | Markdown split pattern v `render/markdown.rs` | Dvě ScrollArea s drag handle, ratio clamp. Ověřený pattern v projektu. |
| Diff rendering (barvy, LayoutJob) | `diff_view.rs` — `render_diff_modal()` | Barvy pro added/removed, `TextFormat` s background. Extrahovat do sdílené funkce. |
| Background I/O | `spawn_task()` v `background.rs` + `mpsc` kanál | Existující pattern pro async operace bez blokování UI. |
| i18n | Fluent `.ftl` soubory v `locales/{cs,en,sk,de,ru}/` | Standardní pattern: definovat klíč v .ftl, volat `i18n.get()`. |

## Existing Code and Patterns

- `src/app/local_history.rs` — Backend service. `take_snapshot` deduplikuje přes xxhash, `get_history` vrací `Vec<HistoryEntry>` seřazený od nejnovějšího. **Chybí:** `get_snapshot_content(rel_path, entry) -> Option<String>` pro načtení obsahu konkrétní verze. **Chybí:** cleanup nemá `max_age` filtrování (jen `max_versions`). Cleanup se nikde nevolá.
- `src/app/ui/background.rs` — Handler v `process_background_events()` řádek 107: `FsChangeResult::LocalHistory(rel_path, content) => ws.local_history.take_snapshot(...)`. **Receiver existuje, ale nikdo do kanálu nepíše.** Sender musí být vytvořen a předán do save logiky.
- `src/app/ui/editor/files.rs` — `save()` metoda (řádek 80): `fs::write(&tab.path, &tab.content)`. Tady patří save hook — po úspěšném zápisu odeslat content do background kanálu. Pozor: metoda nemá přístup k `background_io_tx` — bude potřeba buď předat sender jako parametr, nebo vrátit content pro odeslání z volajícího kódu.
- `src/app/ui/editor/render/tabs.rs` — Tab bar rendering. Selectable labels bez context menu. Pro "Historie souboru" položku je potřeba přidat `r.context_menu()` na tab response — pattern z `file_tree/render.rs` řádek 179.
- `src/app/ui/editor/render/markdown.rs` — Markdown split view s drag handle. `split_axis()` helper, dvě `ScrollArea` s resize logikou. Přesný vzor pro history split view.
- `src/app/ui/editor/diff_view.rs` — AI diff modal. Side-by-side grid s `similar::TextDiff::from_lines()`. Barvy: `bg_added` (zelená), `bg_removed` (červená), `fg_added`, `fg_removed`. Tato logika se dá extrahovat do sdílené utility.
- `src/app/ui/editor/mod.rs` — Definice `Editor` struct a `Tab`. **Chybí:** žádný stav pro "history mode" (aktuálně zobrazená historická verze, index v historii, atd.). Bude potřeba přidat stav buď do `Editor` nebo jako separátní struct.
- `src/app/ui/workspace/state/types.rs` — `FsChangeResult` enum s variantou `LocalHistory(PathBuf, String)`. Připraveno, stačí použít.
- `src/watcher.rs` řádek 204 — `.polycredo` adresář je filtrován z project watcheru. ✓ Ověřeno, history snapshoty nebudou triggerovat watcher eventy.
- `src/app/ui/workspace/state/mod.rs` — `WorkspaceState` drží `local_history: LocalHistory` a `background_io_rx: Option<Receiver<FsChangeResult>>`. Receiver je vždy `None`.

## Constraints

- **Žádné nové runtime závislosti** — `similar` 2.7.0 a `xxhash_rust` už v Cargo.toml.
- **Neblokovat UI vlákno** — save hook musí odeslat content přes kanál, snapshot I/O běží v background handleru (existující pattern).
- **`cargo check` + `./check.sh` musí projít** po každé fázi.
- **Single-process multi-window architektura** — history stav musí být per-workspace (žije ve `WorkspaceState`), ne globální.
- **Save metoda `Editor::save()` nemá přístup k channel tx** — metoda vrací `Option<String>` (error). Buď: (a) přidat `Option<mpsc::Sender<FsChangeResult>>` do `Editor` structu, nebo (b) vrátit z `save()` signal a odeslat z volajícího kódu v workspace. Varianta (b) je čistší — nemíchá I/O concerns do editoru.
- **Binary soubory** — `tab.is_binary` flag existuje. Snapshot se musí přeskočit pro binární taby.
- **i18n: 5 jazyků** (cs, en, sk, de, ru) — každý nový UI text musí mít klíče ve všech 5 ftl souborech.
- **Context menu na tab baru** je nový pattern — dosud existuje jen na editor body a file tree. Tab selectable label v `render/tabs.rs` to podporuje přes `r.context_menu()`.

## Common Pitfalls

- **Snapshot při autosave floods** — Autosave běží po 500ms debounce od posledního edit. Pokud uživatel píše rychle, snapshoty se nebudou množit díky xxhash deduplikaci v `take_snapshot()`. Ale pokud se mění jeden znak, hash bude jiný → nový snapshot. Deduplikace chrání jen proti opakovanému uložení stejného obsahu. **Mitigace:** Retence 50 verzí + 30 dní cleanup je dostatečný limit.
- **Save hook v `save()` vs `save_path()`** — Existují dvě save metody: `save()` (aktivní tab) a `save_path()` (konkrétní tab). Obě musí odeslat snapshot hook, jinak autosave/manual save nebude konzistentní.
- **Split view stav při zavření tabu** — Pokud je otevřený history split view a uživatel zavře tab, stav split view musí být vyčištěn. Zajistit v `close_tab()`.
- **Velké soubory a diff výkon** — `similar::TextDiff::from_lines()` je O(n*d) kde d je edit distance. Pro soubory >10k řádků může být pomalý. **Mitigace:** Diff spočítat jednou při otevření/navigaci historie, ne při každém frame renderu. Cachovat diff výsledek.
- **History split view vs markdown split view konflikt** — Pokud je otevřený .md soubor, editor renderuje markdown split. History mode musí mít přednost — nebo se musí vzájemně vyloučit. Jednodušší: history mode nahradí normální rendering úplně.
- **`background_io_rx` je `Option` a vždy `None`** — Pro propojení bude potřeba vytvořit kanál v `init_workspace()` a uložit sender někam přístupným z workspace save logiky.

## Open Risks

- **Editor dual-pane mode je nový koncept** — Dosud editor renderuje vždy jeden panel (text nebo markdown split). History split view vyžaduje nový rendering path. Riziko: integrace se stávajícím scroll/cursor/LSP state může být komplikovaná. Mitigace: history panely jsou read-only, neinteragují s LSP ani cursor state.
- **Synchronizace scrollu dvou panelů** — Oba panely mohou mít různý počet řádků (pokud se řádky přidaly/odebraly). Sync scroll by vyžadoval diff-aware mapování řádků. **Doporučení:** nezávislý scroll — jednodušší, funkční, žádný UX problém.
- **Cleanup timing** — Cleanup se dosud nikde nevolá. Musí běžet při startu workspace (jednorázově) nebo periodicky. Pokud běží synchronně v init, může zpomalit start. **Doporučení:** spustit v background threadu při startu.
- **Stávající `.polycredo/history/` data** — Kontext říká "testovací relikty, budou smazány". Buď smazat při prvním spuštění nové verze, nebo prostě cleanup vyčistí staré záznamy automaticky. Čistší: smazat obsah v první slice.

## Candidate Requirements (advisory, not auto-binding)

- **CR-1:** Save hook musí pokrýt i `save_path()`, nejen `save()` — jinak autosave cest nevytvoří snapshoty.
- **CR-2:** `cleanup()` musí mít i `max_age` parametr (30 dní) — aktuálně podporuje jen `max_versions`.
- **CR-3:** Diff výsledek cachovat per history entry — nerenderovat `TextDiff::from_lines()` každý frame.
- **CR-4:** History split view musí být read-only — žádná editace, žádná LSP interakce.
- **CR-5:** Binární soubory vyloučit ze snapshot producenta (ověřit `tab.is_binary`).
- **CR-6:** Watcher filter pro `.polycredo/` je ověřen ✓ — neblokuje, ale stojí za regresní test.
- **CR-7:** Chybové stavy (I/O error při snapshot) propagovat do UI toast (per S-3 z backlogu).

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| egui/eframe | — | None found (žádné relevantní egui skills existují) |
| Rust desktop | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` (119 installs) | Available — generický, nepotřebný pro tento scope |

## Sources

- egui `ScrollArea`, `SidePanel`, layout patterns (source: [Context7 egui docs](https://context7.com/emilk/egui))
- `similar` crate pro text diffing — již v projektu, API známé z `diff_view.rs`
- Codebase exploration: `rg`, `find`, targeted reads přes 15+ klíčových souborů
