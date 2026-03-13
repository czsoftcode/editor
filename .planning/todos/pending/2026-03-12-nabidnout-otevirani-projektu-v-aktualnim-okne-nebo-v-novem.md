---
created: 2026-03-12T09:24:21.716Z
title: Nabidnout otevirani projektu v aktualnim okne nebo v novem
area: ui
files:
  - src/app/startup.rs
  - src/app/ui/workspace/menubar/project.rs
  - src/ipc.rs
---

## Problem

Pri otevreni projektu neni uzivateli jasne, zda se ma projekt otevrit v aktualnim okne, nebo v novem. Chybi explicitni volba, ktera by sjednotila chovani mezi startup flow a otevrenim projektu z menu.

## Solution

Pridat jasnou volbu "otevrit v aktualnim okne" vs. "otevrit v novem okne" do relevantniho open-project toku (startup + project menu). Zachovat existujici single-process multi-window architekturu, napojit volbu na IPC/new-window mechanismus a sjednotit UX texty i default.
