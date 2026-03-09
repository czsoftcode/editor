# Architecture Research

**Domain:** Save pipeline and close-safety in desktop editor
**Researched:** 2026-03-09
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │ Editor Tabs  │  │ Settings Modal │  │ Close Dialog   │  │
│  └──────┬───────┘  └──────┬─────────┘  └──────┬─────────┘  │
│         │                 │                   │             │
├─────────┴─────────────────┴───────────────────┴─────────────┤
│                     Save Coordination Layer                  │
├─────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Save Mode + Dirty-State + Close Decision Engine      │  │
│  └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                        Persistence Layer                    │
│  ┌───────────────┐  ┌───────────────┐                      │
│  │ File Writes   │  │ settings.toml │                      │
│  └───────────────┘  └───────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Editor tab state | Drží `modified/dirty` informaci per soubor | Existing tab model in workspace/editor state |
| Save mode config | Source of truth auto vs manual režimu | Settings field + serde default + runtime apply |
| Close guard handler | Rozhodnutí Save/Discard/Cancel při close eventu | Shared handler called from tab-close i app-close paths |

## Recommended Project Structure

```
src/
├── app/ui/editor/                  # editor rendering + tab save flow
├── app/ui/workspace/modal_dialogs/ # settings a confirm dialogy
├── settings.rs                     # persistovaný config model
└── app/mod.rs / workspace close    # app close interception body
```

### Structure Rationale

- **editor/**: dirty-state a save trigger patří k editor tab logice.
- **settings.rs**: save mode musí být persistovaný stejně jako ostatní uživatelská preference.
- **modal_dialogs/**: close confirm dialog držet v již existujícím modal patternu.

## Architectural Patterns

### Pattern 1: Single Source of Truth for Save Mode

**What:** Save režim je držen v jednom poli settings + promítnut do runtime.
**When to use:** Když má režim ovlivnit více částí UI/chování.
**Trade-offs:** Méně driftu, ale nutnost řešit migraci/default.

### Pattern 2: Centralized Close Guard

**What:** Jedna funkce pro vyhodnocení dirty close scénářů.
**When to use:** Když existují 2+ cesty zavření (tab, app, viewport).
**Trade-offs:** Méně duplicit, ale vyžaduje čistý API kontrakt.

### Pattern 3: Explicit User Decision Branching

**What:** Save / Discard / Cancel jako explicitní větve.
**When to use:** Kdykoliv hrozí ztráta neuložené práce.
**Trade-offs:** O jeden dialog víc, ale výrazně bezpečnější UX.

## Data Flow

### Request Flow

```
[User presses Ctrl+S]
    ↓
[Editor Handler] → [Save Dispatcher] → [File Write]
    ↓                                 ↓
[Status update] ← [Result mapped to UI toast/state]
```

### State Management

```
[Settings.save_mode]
    ↓
[Workspace/Editor Runtime]
    ↓
[Save trigger + close guard behavior]
```

### Key Data Flows

1. **Mode change flow:** Settings toggle -> persist -> runtime apply -> UI indicator refresh.
2. **Close guard flow:** Close event -> dirty scan -> decision dialog -> save/discard/cancel action.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 0-1k files/project | Current in-memory dirty tracking is fine |
| 1k-100k files/project | Optimize dirty scan to open tabs only |
| 100k+ files/project | Batch close/save orchestration, async queue visibility |

### Scaling Priorities

1. **First bottleneck:** app-close dirty scan across many tabs.
2. **Second bottleneck:** sequential save latency at app shutdown.

## Anti-Patterns

### Anti-Pattern 1: Implicit save mode fallback

**What people do:** režim není jasně vidět ani persistován.
**Why it's wrong:** uživatel neví, proč se soubor uložil/neuložil.
**Do this instead:** explicitní toggle + label + consistent behavior.

### Anti-Pattern 2: Duplicate close logic per entrypoint

**What people do:** tab close a app close mají vlastní rozdílné rozhodování.
**Why it's wrong:** nekonzistentní chování a regresní bugy.
**Do this instead:** shared close guard handler.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| File system | Direct write + error propagation | Chyby vždy přenést do UI, neignorovat |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| settings.rs ↔ editor runtime | shared state + apply on save | Keep atomic update semantics |
| close event handler ↔ editor tabs | direct state inspection | Use open-tabs dirty scan only |

## Sources

- `.planning/PROJECT.md`
- Existing code layout under `src/app/ui/editor`, `src/app/ui/workspace`, `src/settings.rs`

---
*Architecture research for: save pipeline in desktop editor*
*Researched: 2026-03-09*
