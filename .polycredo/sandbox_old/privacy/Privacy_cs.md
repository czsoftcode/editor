# PolyCredo Editor — Zásady ochrany soukromí

**Poslední aktualizace:** 2026-02-20

---

## Co je PolyCredo Editor

PolyCredo Editor je desktopový editor kódu. Jde o grafické rozhraní pro práci se soubory na vašem stroji a pro spouštění AI CLI nástrojů (jako Claude Code nebo Gemini CLI) v integrovaném terminálu.

---

## Sběr dat

**PolyCredo Editor žádná data neodesílá ani neshromažďuje.**

Konkrétně editor:

- neodesílá váš zdrojový kód, obsah souborů ani cesty k souborům na žádný server
- neobsahuje telemetrii, analytiku ani hlášení chyb
- sám od sebe nekomunikuje s žádným externím API
- nevyžaduje účet ani registraci
- neukládá žádná data mimo váš stroj

Veškerý stav, který editor ukládá (otevřené session, nedávné projekty, nastavení), se ukládá lokálně do `~/.config/polycredo-editor/`.

---

## AI nástroje

PolyCredo Editor se integruje s AI CLI nástroji třetích stran, jako je **Claude Code** (Anthropic) a **Gemini CLI** (Google). Tyto nástroje běží jako samostatné procesy v integrovaném terminálu editoru — stejně jako kdybyste je spustili v jakémkoliv jiném terminálu.

**Editor nekontroluje, nezachytává ani nepřeposílá žádná data, která tyto nástroje odesílají nebo přijímají.**

Když AI nástroj v PolyCredo Editoru používáte:

- nástroj komunikuje přímo se servery svého poskytovatele (Anthropic, Google apod.)
- platí zásady ochrany soukromí a podmínky použití daného poskytovatele
- za to, co s AI nástrojem sdílíte, zodpovídáte vy — stejně jako kdybyste ho spustili v samostatném terminálu

Role PolyCredo Editoru je ekvivalentní terminálovému emulátoru — zobrazuje výstup, obsah nezpracovává ani nepřeposílá.

### Offline / lokální AI

Pokud používáte lokálně běžící model (např. Ollama + Aider), žádná data neopouštějí váš stroj. PolyCredo Editor podporuje jakýkoliv AI nástroj, který běží jako CLI proces — včetně plně lokálních řešení.

---

## Rozdělení zodpovědnosti

| Kdo | Zodpovědnost |
|---|---|
| **Autor PolyCredo Editoru** | Zajistit, aby editor sám neodesílal žádná data bez explicitní akce uživatele |
| **Uživatel** | Volba AI nástrojů a přijetí jejich podmínek použití |
| **Poskytovatel AI nástroje** (Anthropic, Google, …) | Nakládání s daty podle vlastních zásad ochrany soukromí |

---

## Cloudové funkce

Případné budoucí cloudové funkce (synchronizace nastavení, sdílení snippetů apod.) budou:

- **opt-in** — ve výchozím stavu vypnuté
- jasně zdokumentované — co odesílají a kam
- zcela volitelné — editor funguje plnohodnotně i bez nich

---

## Kontakt

Dotazy a hlášení problémů: [https://github.com/czsoftcode/editor/issues](https://github.com/czsoftcode/editor/issues)
