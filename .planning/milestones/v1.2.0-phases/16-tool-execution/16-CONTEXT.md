# Phase 16: Tool Execution - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

AI muze cist/editovat soubory a spoustet prikazy s uzivatelem schvalenym approval workflow. Zahrnuje: automaticky editor kontext pri kazde zprave, nativni Ollama tool calling, file read/write/replace tooly, command execution, approval UI (approve/deny/always), ask-user tool, bezpecnostni guardraily (blacklist, audit log, scrubbing). i18n patri do Phase 17.

</domain>

<decisions>
## Implementation Decisions

### Kontext injection
- Kontext se pripoji jako system message pri KAZDE zprave — AI vzdy vidi aktualni stav
- Stavajici AiContextPayload (open_files, build_errors, active_file, cursor, git_branch, git_status, cargo_toml_summary) + PRIDAT:
  - Terminal output: poslednich N radku z aktivniho terminalu
  - LSP diagnostiky: warnings/hints z rust-analyzeru (pokud bezi)
- Obsah aktivniho souboru: pouzit RSG indexaci — nejdrive projit index, najit odpovidajici kontext a poslat +-50 radku. Pokud se netrefi, posilat po 200 radcich (~10 000 tokenu) dokud nenajde spravny kontext
- write_file celeho souboru jen u noveho souboru nebo v nejnutnejsich pripadech — preferovat replace

### Diff preview a file edity
- Inline unified diff (jako git diff) — existujici render_diff_or_markdown v approval.rs
- 3 radky kontextu pred a po zmene (git diff -U3 standard)
- Po schvaleni: okamzity zapis na disk + watcher detekuje zmenu a reloada tab
- Rozlisovat novy soubor vs. prepsani existujiciho:
  - Novy soubor = nizsi riziko, jednodussi approval
  - Prepsani existujiciho = varovani uzivateli + diff preview

### Command execution
- VZDY vyzadovat approval + blacklist zakazanych prikazu
- Blacklist (uplne zakazane): rm -rf /, sudo, shutdown, reboot, mkfs, dd, format
- Sitove prikazy (curl, wget, nc, ssh, scp, rsync, telnet) = approval s extra varovanim "Sitovy prikaz — data mohou opustit pocitac"
- Timeout: 120 sekund
- Vystup inline v chatu jako code block pod tool call zpravou
- Working directory: vzdy project root (ws.root_path)

### Tool call orchestrace
- Nativni Ollama tools API — poslat tool deklarace pres Ollama 'tools' parametr, Ollama vrati tool_calls
- StreamEvent::ToolCall uz existuje v provider.rs — vykonat v Rustu, vysledek zpet jako tool message
- Jeden tool call na zpravu — AI musi cekat na schvaleni a uzivatelovu reakci pred dalsim tool callem
- Zobrazeni v chatu: kompaktni tool blok s ikonou + nazev toolu + parametry, rozbalitelny pro detaily
- Approval rozdeleni:
  - Automaticky (bez approval): read_project_file, list_project_files, search_project, semantic_search, store_scratch, retrieve_scratch, store_fact, retrieve_fact, list_facts, delete_fact
  - S approval: write_file, replace, exec
  - Vlastni UI: ask_user, announce_completion

### Bezpecnost — souborovy blacklist
- Respektovat .gitignore — ignorovane soubory se NEPOSILAJI do kontextu a AI je NESMI cist
- Extra blacklist v AI Settings (globalni pro vsechny projekty): glob patterny pro citlive soubory
- Vychozi patterny: .env*, *.pem, *.key, id_rsa*, credentials.*, secrets.*, *.pfx, *.p12

### Bezpecnost — path traversal sandbox
- Vsechny cesty se kanonizuji (resolve symlinks) a MUSI zacinat pod ws.root_path
- Jakykoli pokus o unik (../, symlink ven, absolutni cesta mimo projekt) = okamzite zamitnuti + audit log zaznam
- Platí pro VSECHNY file tooly (read, write, replace, list, search)

### Bezpecnost — secrets scrubbing
- Automaticke maskovani secrets pred odesilanim do AI kontextu
- Regex detekce patternu: KEY=, PASSWORD=, SECRET=, TOKEN=, API_KEY=, DB_PASSWORD=, *_URL s credentials
- Hodnoty se nahradi [REDACTED]
- Funguje pro .env, JSON, YAML, TOML konfigurace
- Stejny filtr se aplikuje na exec vystup pred odesilanim zpet AI

### Bezpecnost — rate limiting
- Max 50 write/replace + 20 exec za konverzaci
- Cteci tooly bez limitu
- Po dosazeni limitu AI musi ukoncit a uzivatel zacina novou konverzaci

### Bezpecnost — audit log
- Kompletni audit log: kazdy tool call, approval rozhodnuti, odeslany kontext
- Logovani do .polycredo/ai-audit.log s casovou znackou
- Format: [timestamp] [tool_name] [approval_status] [details]

### Claude's Discretion
- Presny regex patterny pro secrets scrubbing
- Format audit logu (JSON vs. plain text)
- Jak zobrazit rate limit warning v UI
- Presna ikonografie pro tool blok typy (read/write/exec)
- Jak zobrazit LSP diagnostiky v kontextu (format)
- Jak extractovat terminal output (kolik radku, ktery terminal)

</decisions>

<specifics>
## Specific Ideas

- "Bezpecnost je na prvnim miste — predstav si Pentagon, statni spravu nebo ucetni program s daty klientu"
- RSG indexace pro chytre odesilani kontextu — ne cely soubor, ale relevantni casti
- write_file POUZE pro nove soubory — existujici soubory vzdy pres replace
- Sitove prikazy s extra varovanim — data mohou opustit pocitac
- Kompletni audit trail pro forenzni analyzu

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tools.rs`: 14 tool deklaraci uz existuje (read_project_file, write_file, replace, exec, search_project, semantic_search, ask_user, announce_completion, store/retrieve_scratch, store/retrieve/list/delete_fact)
- `approval.rs`: Approval UI (approve/deny/always) + ask_user UI + diff rendering — adaptovat z WASM na nativni
- `AiManager::generate_context()` (ai/mod.rs): Generuje AiContextPayload s open files, build errors, git status, cursor, cargo.toml
- `StreamEvent::ToolCall` (provider.rs): Varianta uz existuje s id, name, arguments
- `AiToolDeclaration` (types.rs): Typ pro tool deklarace s name, description, parameters
- `AiMessage` (types.rs): Uz ma tool_call_name, tool_call_id, tool_result_for_id, tool_is_error pole
- `render_diff_or_markdown` (approval.rs): Diff rendering s +/- barevnymi radky

### Established Patterns
- mpsc::Receiver pro async vysledky (build_error_rx, git_status_rx, ollama_check_rx)
- PluginApprovalResponse enum (Approve/ApproveAlways/Deny) v types.rs
- pending_plugin_approval tuple v WorkspaceState — adaptovat na nativni tool approval
- cancellation_token (Arc<AtomicBool>) pro preruseni
- Toast (AppAction::ShowToast) pro user feedback

### Integration Points
- `OllamaProvider.stream_chat()`: Pridat tools parametr do /api/chat requestu
- `OllamaProvider.capabilities()`: Zmenit supports_tools na true
- `background.rs`: Zpracovavat StreamEvent::ToolCall — spustit nativni executor
- `ai_chat/render.rs`: Pridat rendering kompaktnich tool bloku
- `ai_chat/logic.rs`: Tool call → approval → execute → send result zpet loop
- `modal_dialogs/settings.rs`: Pridat AI blacklist patterns do settings
- `AiContextPayload` (types.rs): Pridat terminal_output a lsp_diagnostics pole

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 16-tool-execution*
*Context gathered: 2026-03-06*
