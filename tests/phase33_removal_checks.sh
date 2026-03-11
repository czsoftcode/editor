#!/usr/bin/env bash
set -euo pipefail

mode="${1:-all}"

check_task1() {
  test ! -d src/app/ai_core
  test ! -d src/app/ui/terminal/ai_chat
  ! rg -n "\\bai_core\\b|ui/terminal/ai_chat" src/app >/dev/null
}

check_task2() {
  ! rg -n "show_ai_chat|tool_executor|run_agent\\s*==\\s*\"ai_chat\"|FocusedPanel::AiChat" src/app >/dev/null
  ! rg -n "\\bws\\.ai\\b" src/app >/dev/null
}

check_task3() {
  ! rg -n "cli-chat|cli-tool" locales >/dev/null
}

case "$mode" in
  task1)
    check_task1
    ;;
  task2)
    check_task2
    ;;
  task3)
    check_task3
    ;;
  all)
    check_task1
    check_task2
    check_task3
    ;;
  *)
    echo "Unknown mode: $mode" >&2
    exit 2
    ;;
esac
