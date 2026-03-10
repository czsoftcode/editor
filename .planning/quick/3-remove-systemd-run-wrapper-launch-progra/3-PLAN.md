---
phase: quick
plan: 3
type: execute
wave: 1
depends_on: []
files_modified:
  - run_limited.sh
  - packaging/deb/wrapper.sh
autonomous: true
requirements: [QUICK-3]
must_haves:
  truths:
    - "run_limited.sh spousti cargo run primo bez systemd-run"
    - "wrapper.sh spousti binarku primo bez systemd-run"
    - "Oba skripty nastavuji RAYON_NUM_THREADS a TOKIO_WORKER_THREADS"
    - "Oba skripty vypocitavaji CPU/RAM limity pro env promenne"
  artifacts:
    - path: "run_limited.sh"
      provides: "Dev launcher bez systemd-run"
    - path: "packaging/deb/wrapper.sh"
      provides: "Deb package launcher bez systemd-run"
  key_links: []
---

<objective>
Odstranit systemd-run wrapper z obou spoustecich skriptu a spoustet program primo.

Purpose: systemd-run pridava zbytecnou zavislost a komplexitu. Program se ma spoustet primo s env promennymi pro omezeni vlaken.
Output: Zjednodusene run_limited.sh a packaging/deb/wrapper.sh bez systemd-run.
</objective>

<execution_context>
@/home/stkremen/.claude/get-shit-done/workflows/execute-plan.md
@/home/stkremen/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@run_limited.sh
@packaging/deb/wrapper.sh
</context>

<tasks>

<task type="auto">
  <name>Task 1: Odstranit systemd-run z run_limited.sh</name>
  <files>run_limited.sh</files>
  <action>
Zjednodusit run_limited.sh:
- Zachovat vypocet TOTAL_CORES, LIMIT_CORES a nastaveni RAYON_NUM_THREADS + TOKIO_WORKER_THREADS
- Odstranit CPU_QUOTA promennou (pouzivala se jen pro systemd-run CPUQuota)
- Odstranit TOTAL_KB, LIMIT_KB, LIMIT_MB promenne (pouzivaly se jen pro systemd-run MemoryMax)
- Odstranit `command -v systemd-run` podminku a cely if/else blok
- Nahradit spustenim: `exec cargo run -- --new-instance "$@"`
- Upravit echo vypis — informovat jen o CPU limitu (pocet vlaken), RAM limit uz neni vynucovan
- Zachovat shebang a uvodni komentare (aktualizovat popis — zmenit na "omezeni vlaken")
  </action>
  <verify>
    <automated>bash -n run_limited.sh && grep -q 'exec cargo run' run_limited.sh && ! grep -q 'systemd-run' run_limited.sh && grep -q 'RAYON_NUM_THREADS' run_limited.sh</automated>
  </verify>
  <done>run_limited.sh neobsahuje systemd-run, spousti cargo run primo, nastavuje RAYON/TOKIO env promenne</done>
</task>

<task type="auto">
  <name>Task 2: Odstranit systemd-run z packaging/deb/wrapper.sh</name>
  <files>packaging/deb/wrapper.sh</files>
  <action>
Zjednodusit packaging/deb/wrapper.sh:
- Zachovat vypocet TOTAL_CORES, LIMIT_CORES a nastaveni RAYON_NUM_THREADS + TOKIO_WORKER_THREADS
- Odstranit CPU_QUOTA promennou (pouzivala se jen pro systemd-run CPUQuota)
- Odstranit TOTAL_KB, LIMIT_KB, LIMIT_MB promenne (pouzivaly se jen pro systemd-run MemoryMax)
- Odstranit `command -v systemd-run` podminku, cely if blok a fallback komentar
- Nahradit jednoduchym: `exec /usr/lib/polycredo-editor/polycredo-editor-bin "$@"`
- Aktualizovat uvodni komentare — zmenit popis na "omezeni vlaken pro Rust runtime"
  </action>
  <verify>
    <automated>bash -n packaging/deb/wrapper.sh && grep -q 'exec /usr/lib/polycredo-editor/polycredo-editor-bin' packaging/deb/wrapper.sh && ! grep -q 'systemd-run' packaging/deb/wrapper.sh && grep -q 'RAYON_NUM_THREADS' packaging/deb/wrapper.sh</automated>
  </verify>
  <done>wrapper.sh neobsahuje systemd-run, spousti binarku primo, nastavuje RAYON/TOKIO env promenne</done>
</task>

</tasks>

<verification>
- Zadny ze souboru neobsahuje retezec "systemd-run"
- Oba soubory jsou syntakticky validni bash skripty (bash -n)
- Oba soubory exportuji RAYON_NUM_THREADS a TOKIO_WORKER_THREADS
- run_limited.sh spousti `cargo run`
- wrapper.sh spousti `/usr/lib/polycredo-editor/polycredo-editor-bin`
</verification>

<success_criteria>
- systemd-run kompletne odstranen z obou skriptu
- Skripty spousteji programy primo pres exec
- Env promenne pro omezeni vlaken zachovany
</success_criteria>

<output>
After completion, create `.planning/quick/3-remove-systemd-run-wrapper-launch-progra/3-SUMMARY.md`
</output>
