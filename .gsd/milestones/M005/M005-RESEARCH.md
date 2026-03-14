# M005: Vylepšení Project Search — Research

**Date:** 2026-03-13

## Summary

Aktuální project search je minimální: modální input dialog → background thread s `line.to_lowercase().contains(&q)` → výsledkový modal se seznamem `selectable_label` bez zvýraznění, kontextu nebo filtrování. Celý flow je 353 řádků v `search_picker.rs` + 28 řádků typů v `types.rs`. Search engine nezná regex, case-sensitivity, whole-word matching ani file type filtr. Replace v projektu neexistuje vůbec.

Dobrá zpráva: `regex = "1"` je již v Cargo.toml, `globset = "0.4"` taky. Projekt má zavedené vzory pro všechno, co budeme potřebovat — LayoutJob s `TextFormat` pro barevné zvýraznění (diff_view.rs, history/mod.rs), background thread s cancel epoch a mpsc kanálem (search_picker.rs), Toast systém pro chybové hlášky (types.rs), `take_snapshot()` pro local history (local_history.rs), a `ProjectIndex::get_files()` pro listing souborů. Riziko je primárně v UX komplexitě: kombinace regex togglerů, zvýraznění matchů ve výsledcích, kontextových řádků, a replace preview dialogu v egui.

**Primární doporučení:** Rozdělit na 4 slicí seřazené dle rizika: (1) search engine s togglery a testy, (2) vylepšený výsledkový dialog se zvýrazněním a kontextem, (3) file type filtr, (4) replace s preview a local history. Engine (S01) je základ pro vše ostatní. Rendering výsledků (S02) je nejvyšší UX riziko. Filtr (S03) je nezávislý. Replace (S04) je nejkomplexnější ale staví na předchozích.

## Recommendation

Začít search enginem (čistá logika, žádné UI, plně testovatelná) a ověřit, že `RegexBuilder` s `case_insensitive()` + word boundary wrapping pokrývá všechny toggle kombinace. Tím se odblokuje vše ostatní. Zvýraznění matchů ve výsledcích je hlavní UX riziko — egui nemá inline rich text v `selectable_label`, takže výsledky budou muset používat `Label::new(LayoutJob)` uvnitř klikatelného kontejneru (pattern z history view). Replace jako poslední — je to nejkomplexnější slice a závisí na funkčním enginu + preview renderingu.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Regex matching s togglery | `regex::RegexBuilder::new().case_insensitive(flag).build()` | Zvládá case-insensitive, `\b` word boundary wrapping, capture groups pro replace. Kompilace na nevalidní pattern vrací `Err` — přímý inline error do UI. |
| Whole-word boundary | `\b` prefix/suffix kolem escaped query v regex | `regex::escape()` + `\b` wrapping. Pro regex mode se `\b` přidá jen když je whole-word toggle ON. |
| File type filtrování | `globset::Glob::new("*.rs")` | Již v Cargo.toml (`globset = "0.4"`), nepoužívaný. Pattern matching přes glob je jednodušší a robustnější než ruční `path.extension() == "rs"`. |
| Zvýrazněné texty v egui | `egui::text::LayoutJob::append()` s `TextFormat { background, color }` | Zavedený pattern v diff_view.rs a history/mod.rs. `Label::new(job)` renderuje rich text. |
| Background search s cancellation | `mpsc::channel` + `Arc<AtomicU64>` cancel epoch | Existující pattern v `run_project_search()`. Rozšířit o streamování per-soubor místo single-shot. |
| Local history snapshots | `LocalHistory::take_snapshot()` | Existující API, vrací `Result<Option<PathBuf>>`. Volat před replace zápisem. |
| I/O error toast | `Toast::error()` + `ws.toasts.push()` | Zavedený pattern. Replace chyby reportovat přes toast. |

## Existing Code and Patterns

- `src/app/ui/search_picker.rs` — **Kompletní přestavba.** 353 řádků. `render_project_search_dialog()` je prostý modal s jedním inputem. `run_project_search()` je background thread s `line.to_lowercase().contains()`. `poll_and_render_project_search_results()` zobrazuje výsledky jako flat `selectable_label` seznam. `collect_project_files()` duplikuje logiku z ProjectIndex — mělo by se nahradit voláním `ProjectIndex::get_files()` (už se tak dělá v dispatch kódu).
- `src/app/ui/workspace/state/types.rs:53-79` — **Rozšířit.** `SearchResult` má jen `file`, `line`, `text`. Potřeba přidat `match_ranges: Vec<(usize, usize)>` pro byte rozsahy matchů v textu řádku, a `context_before`/`context_after` pro kontextové řádky. `ProjectSearch` potřeba přidat toggle stavy (`use_regex`, `case_sensitive`, `whole_word`), `file_filter: String`, a replace-related data.
- `src/app/ui/editor/search.rs` — **Referenční pattern pro in-file search.** `update_search()` dělá case-insensitive substring match přes char_indices. `apply_search_highlights()` modifikuje LayoutJob sections background barvy pro match zvýraznění. **Neopravovat** — in-file search vylepšení je out of scope, ale engine logika z project search se může v budoucnu sdílet.
- `src/app/ui/editor/diff_view.rs:75-115` — **Pattern pro LayoutJob.** `LayoutJob::append()` s `TextFormat { font_id, color, background }` pro vytváření barevného textu. `Label::new(job).wrap_mode(TextWrapMode::Extend)` pro rendering.
- `src/app/ui/workspace/history/mod.rs:232-262` — **Pattern pro apply_diff_backgrounds.** Binary search v line_offsets pro mapování byte_range → řádek. Reusovatelný koncept pro highlight matchů ve výsledcích.
- `src/app/ui/workspace/history/mod.rs:587-594` — **Pattern pro Label::new(LayoutJob).** Highlighter → clone → modify → render. Přesně to, co potřebujeme pro zvýrazněné výsledky.
- `src/app/ui/workspace/mod.rs:534-539` — **Dispatch bod.** `render_project_search_dialog()` → `poll_and_render_project_search_results()` → `open_file_in_ws()` + `jump_to_location()`. Rozšíří se o replace flow.
- `src/app/ui/workspace/index.rs` — **ProjectIndex::get_files()** vrací `Arc<Vec<PathBuf>>`. Filtrování dle přípony se přidá buď sem (nová metoda `get_files_filtered()`) nebo v search threadu (jednodušší, bez blokování indexu).
- `src/app/local_history.rs:67` — **take_snapshot()** API pro pre-replace snapshoty. Signature: `(&mut self, relative_file_path: &Path, content: &str) -> Result<Option<PathBuf>>`.
- `src/app/types.rs:168-194` — **Toast API.** `Toast::error(msg)`, `Toast::info(msg)`, `ws.toasts.push()`. 4s expiration.
- `locales/*/ui.ftl` — 5 jazyků (cs, en, sk, de, ru). 7 existujících `project-search-*` klíčů. Potřeba ~15 nových klíčů (toggle labely, replace UI, error messages, context labels).

## Constraints

- **Žádné nové runtime závislosti** — `regex` a `globset` jsou již v Cargo.toml.
- **UI vlákno nesmí blokovat** — search a replace v background threadu. Stávající `run_project_search()` pattern s `mpsc::channel` + `AtomicU64` cancel epoch je základ.
- **`cargo check` + `./check.sh` musí projít** — fmt, clippy -D warnings, 172+ testů.
- **Regex error = inline error v dialogu, ne panic** — `Regex::new()` vrací `Result`. Error message zobrazit vedle inputu.
- **Replace musí volat `take_snapshot()` PŘED zápisem** — `LocalHistory` není Send, takže snapshot se musí vzít na main threadu (nebo předat snapshot logiku do threadu přes PathBuf a content).
- **Replace error handling: per-file, ne atomic** — pokud zápis jednoho souboru selže, reportovat přes toast a pokračovat. Uživatel vidí, které soubory se nepodařilo změnit.
- **`Ctrl+Shift+F` dispatch již funguje** přes centrální keymap z M004 (`CommandId::ProjectSearch`). Žádné změny v keymap.rs potřeba.
- **`LocalHistory` drží `&mut self`** — volání `take_snapshot()` vyžaduje mutable borrow. V replace flow se musí snapshoty volat sekvenčně v workspace handleru (ne v background threadu).

## Common Pitfalls

- **Regex error UX** — Nevalidní regex pattern nesmí crashovat aplikaci. `RegexBuilder::build()` vrací `Err(regex::Error)` s popisnou zprávou. Zobrazit inline pod inputem červeně. Netriggernout search pokud build selže. Testovat edge cases: prázdný pattern, `(`, `*`, `(?P<>)`.
- **Word boundary + non-regex mode** — Pokud uživatel nezadá regex ale chce whole-word, query se musí obalit do `\bESCAPED_QUERY\b`. `regex::escape()` escapuje speciální znaky. Ale pozor: `\b` nedělá to co uživatel čeká u non-ASCII slov (závisí na Unicode word boundary definici). Pro většinu programátorského use-case (identifikátory) to stačí.
- **Streamování výsledků přes mpsc** — Aktuální kód posílá `Vec<SearchResult>` jednorázově. Pro streamování změnit na posílání po dávkách (po souboru nebo po N výsledcích). UI musí akumulovat výsledky z více `try_recv()` volání. Nový `rx` typ: `mpsc::Receiver<Vec<SearchResult>>` kde každý recv je dávka, ne finální výsledek. Potřeba rozlišit "ještě běží" vs "hotovo" — buď sentinel value nebo druhý kanál.
- **LayoutJob pro výsledky — klikatelnost** — `egui::Label::new(LayoutJob)` není samo o sobě klikatelné. Řešení: `ui.add(Label::new(job).sense(Sense::click()))` nebo wrap do `ui.allocate_ui()` s manuálním response. Alternativa: `Button::new(LayoutJob)` — otestovat co egui podporuje. Fallback: vrátit se k `selectable_label` s prostým textem ale přidat barevné LayoutJob pro kontextový řádek.
- **Kontextové řádky — deduplikace** — Pokud dva matche jsou ≤4 řádky od sebe, kontextové bloky se překrývají. Sloučit je do jednoho bloku (grep -C styl). Bez sloučení se stejné řádky opakují — matoucí UX.
- **Replace s capture groups** — `Regex::replace_all(&content, replacement)` zpracuje `$1`/`$2`/`${name}` automaticky. Ale nevalidní backreference (např. `$99` když je jen 2 skupiny) se interpretují jako literal text — dokumentovat v UI nebo validovat.
- **Replace atomicita** — Context ŘÍKÁ "ne atomic all-or-nothing", ale per-file error reporting. Snapshot PŘED zápisem. Pokud `fs::write()` selže, snapshot zůstane (není destruktivní). Pokud snapshot selže (disk full), nespouštět zápis a reportovat.
- **File type filtr parsování** — Glob pattern `*.rs` je intuitivní. Ale co `src/*.rs`? `globset::Glob` matchuje celou cestu, ne jen příponu. Rozhodnout: filtrovat jen podle přípony (jednodušší) nebo plný glob (flexibilnější). Doporučení: plný glob přes `globset`, ale hint text ukazuje příklady (`*.rs`, `*.toml`, `src/**/*.rs`).

## Open Risks

- **egui klikatelný LayoutJob** — egui `Label` s `LayoutJob` nemusí mít `sense(Sense::click())` ve všech verzích. Potřeba ověřit v runtime. Fallback: custom widget s `allocate_response()` + `paint_galley()`.
- **Výkon regex na tisících souborů** — `regex` crate je rychlá (DFA), ale kompilace regex patternu probíhá jednou na search. Na projektu s 5000 soubory by search měl trvat < 2s. Monitorovat a případně přidat progress bar.
- **`LocalHistory` borrow v replace flow** — `take_snapshot()` potřebuje `&mut LocalHistory`, ale replace results přicházejí z background threadu. Snapshoty se musí brát v main threadu v sekvenčním loopu. Pokud replace mění 100 souborů, to je 100 volání `take_snapshot()` + `fs::write()` — potenciálně sekundy blokování UI. Řešení: buď přijmout krátký freeze (100 souborů = ~100ms), nebo přesunout snapshot logiku do threadu (vyžaduje refactor LocalHistory na thread-safe nebo extrakci snapshot funkce jako standalone).
- **Replace preview UX** — Zobrazení diff všech 100 souborů v jednom dialogu je nepřehledné. Řešení: collapsible per-file sekce s checkboxy. Každá sekce ukáže starý → nový text pro daný soubor. Limit na viewport height + scroll.

## Candidate Requirements

Následující by měly být formalizovány jako requirements v REQUIREMENTS.md:

| ID | Class | Description | Notes |
|----|-------|-------------|-------|
| R016 | core-capability | Regex search engine s RegexBuilder, case-insensitive/sensitive toggle, whole-word toggle | Table stakes pro programátorský editor |
| R017 | primary-user-loop | Zvýrazněné matchující části ve výsledcích přes LayoutJob | Bez zvýraznění uživatel nevidí kde přesně je match |
| R018 | primary-user-loop | Kontextové řádky (±2) kolem matchujícího řádku se sloučením blízkých matchů | Standard v grep/VS Code |
| R019 | primary-user-loop | File type filtr (glob pattern) | Omezení scope hledání |
| R020 | primary-user-loop | Project-wide replace s preview a per-file checkboxy | Destruktivní operace vyžaduje preview |
| R021 | failure-visibility | Regex error zobrazený inline v dialogu | Ne panic, ne toast — inline pod inputem |
| R022 | failure-visibility | Replace I/O error reporting přes toast per-file | Pokračovat s dalšími soubory při chybě jednoho |
| R023 | core-capability | Local history snapshot před replace | Záchranná síť pro undo |
| R024 | launchability | i18n pro všechny nové UI prvky (5 jazyků) | Konzistence s existujícím i18n |
| R025 | primary-user-loop | Inkrementální streamování výsledků | UX — uživatel vidí výsledky ihned, ne po dokončení celého searche |

**Capture groups v replace** (`$1`, `$2`) — ponechat jako advisory, ne requirement. `Regex::replace_all()` je podporuje automaticky, ale explicitní UX (dokumentace, validace) by rozšiřoval scope. Pokud to regex crate dělá "zadarmo", prostě to funguje.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Rust desktop (eframe/egui) | `bobmatnyc/claude-mpm-skills@rust-desktop-applications` | available (120 installs) — general Rust desktop, ne egui-specific |
| Rust | `petekp/agent-skills@rust` | available (22 installs) — general Rust |
| regex | none specific | žádný relevantní skill pro regex crate |

Žádný z nalezených skillů není dostatečně specifický pro egui nebo pro tento typ práce. Projekt má vlastní zavedené vzory, které jsou spolehlivější než generické rady.

## Sources

- `regex` crate API: `RegexBuilder::new().case_insensitive().build()`, `Regex::find_iter()`, `Regex::replace_all()`, `regex::escape()` (source: [rust-lang/regex](https://github.com/rust-lang/regex))
- egui LayoutJob pattern: `LayoutJob::append(text, leading, TextFormat { color, background })` → `Label::new(job)` (source: existující kód v diff_view.rs, history/mod.rs)
- globset crate: `Glob::new("*.rs").compile_matcher()` pro file type filtrování (source: Cargo.toml dependency)
- Existující search pattern: `run_project_search()` s mpsc + AtomicU64 cancel epoch (source: search_picker.rs)
