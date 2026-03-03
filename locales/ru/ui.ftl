# Общие элементы интерфейса пользователя

## Панели
panel-files = Файлы
panel-files-sandbox = Файлы (Sandbox)
btn-tree-project = Проект
btn-tree-sandbox = Sandbox
panel-runners = Spouštěče
panel-build = Build
panel-git = Git
panel-build-errors =
    { $count ->
        [one] Ошибка (1)
        [few] Ошибки ({ $count })
        [many] Ошибок ({ $count })
       *[other] Ошибок ({ $count })
    }

## Кнопки сборки
btn-build = ▶ Build
btn-build-sandbox-on = Sandbox ВКЛ
btn-build-sandbox-off = Sandbox ВЫКЛ
hover-build-sandbox = Переключение между выполнением в корне проекта и в ИИ-песочнице
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Create .deb
hover-create-deb-disabled = Cannot create package in sandbox mode. Switch to Sandbox OFF.
hover-build-menu-disabled = Building is disabled in Sandbox ON mode or if there are unpromoted files in the sandbox.
btn-run-profile = ▶ Запустить...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Редактировать
runner-none = Профили не определены.

menu-build-windows = Windows

## Dependency Wizard
dep-wizard-title = Dependency Installation Wizard
dep-wizard-install-question = Do you want to download and install { $tool } to { $path }?
dep-wizard-install-cmd-question = Do you want to start the installation of { $tool } using a system command?
dep-wizard-btn-install = Install
dep-wizard-btn-run-cmd = Start Installation (requires sudo)
dep-wizard-status-downloading = Downloading...
dep-wizard-status-running = Installing...
dep-wizard-status-success = Installation successful!
dep-wizard-status-error = Installation error: { $error }

command-name-install-nsis = Install NSIS
command-name-install-rpm = Install rpm-build (dnf)
command-name-install-generate-rpm = Install cargo-generate-rpm
command-name-install-appimage = Install cargo-appimage
command-name-install-flatpak = Install flatpak-builder
command-name-install-snap = Install snapcraft
command-name-install-deb-tools = Install Debian Build Tools
command-name-install-freebsd-target = Установить FreeBSD Target (rustup)
command-name-install-cross = Установить cross (кросс-компиляция)
command-name-install-fpm = Установить fpm (gem install fpm)
command-name-install-podman = Установить Podman (контейнер для cross)
command-name-install-appimagetool = Install appimagetool
command-name-install-windows-target = Install Windows Target (rustup)
command-name-install-xwin = Install cargo-xwin
command-name-install-clang = Install Clang (LLVM)
command-name-install-lld = Install LLD (Linker)

dep-wizard-xwin-desc = The cargo-xwin tool is required for cross-compiling for Windows MSVC from Linux.
dep-wizard-generate-rpm-desc = The cargo-generate-rpm tool is required to create an .rpm package directly from your Rust project.
dep-wizard-appimage-desc = The cargo-appimage tool is required to create a portable AppImage package directly from your Rust project.
dep-wizard-flatpak-desc = The flatpak-builder tool is required to build and package the application into the Flatpak format.
dep-wizard-snap-desc = The snapcraft tool is required to create Snap packages for Ubuntu and other distributions.
dep-wizard-lxd-desc = LXD — контейнерная система, необходимая snapcraft для сборки Snap-пакетов. Добавляет пользователя в группу lxd и инициализирует LXD.
dep-wizard-deb-desc = System tools like dpkg-dev, build-essential, and fakeroot are required to create .deb packages.
dep-wizard-freebsd-target-desc = Стандартная библиотека Rust для x86_64-unknown-freebsd необходима для кросс-компиляции под FreeBSD.
dep-wizard-cross-desc = Инструмент cross обеспечивает кросс-компиляцию для FreeBSD и других платформ через Docker/Podman-контейнеры.
dep-wizard-fpm-desc = fpm (Effing Package Manager) позволяет создавать нативные FreeBSD .pkg пакеты из Linux.
dep-wizard-podman-desc = Podman — контейнерный движок, необходимый инструменту cross для кросс-компиляции. Альтернатива Docker без фонового демона.
dep-wizard-clang-desc = Clang compiler is required for building native C/C++ dependencies for Windows.
dep-wizard-lld-desc = LLD linker is required for linking Windows binaries on Linux.
dep-wizard-windows-target-desc = Rust standard library for x86_64-pc-windows-msvc is required for compilation.
dep-wizard-nsis-desc = NSIS is required to create Windows installers.
dep-wizard-rpm-desc = The rpmbuild utility is required to create .rpm packages.
dep-wizard-appimagetool-desc = The appimagetool utility is required for final AppImage bundling.

menu-build-fedora = Fedora
menu-build-debian = Debian / Ubuntu
menu-build-freebsd = FreeBSD
menu-build-freebsd-pkg = Собрать .pkg (FreeBSD)
menu-build-flatpak-sub = Flatpak
menu-build-snap-sub = Snap
menu-build-appimage-sub = AppImage
menu-build-deb = Собрать .deb
menu-build-rpm = Собрать .rpm
menu-build-flatpak = Собрать Flatpak bundle
menu-build-snap = Собрать Snap пакет
menu-build-appimage = Собрать .AppImage
menu-build-exe = Собрать .exe (Windows)
menu-build = Build


## Операции Git
git-add-all = git add .
git-commit = git commit -m "..."
git-push = git push
git-status = git status
git-diff = git diff
git-checkout-file = git checkout (файл)
git-checkout-branch = git checkout (ветка)
git-pull = git pull
git-reset-hard = git reset --hard
hover-git-disabled-sandbox = Операции Git отключены до тех пор, пока не будут разрешены все изменения в песочнице (используйте кнопку «Проверить изменения» или «Принять все» на желтой панели).

## Строка состояния
statusbar-line-col = Строка { $line }, Столбец { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Не сохранено
statusbar-saving = Сохранение…
statusbar-saved = Сохранено
statusbar-lsp-initializing = LSP инициализируется...
statusbar-filetype-plain = Обычный текст

## Вкладки редактора
tab-unsaved-indicator = ●
tab-deleted-indicator = ⚠

## Поиск и замена
search-label = Найти:
replace-label = Заменить:
search-replace-expand = Заменить…
search-placeholder = Поиск…
replace-placeholder = Замена…
search-prev = ▲
search-next = ▼
search-replace-one = Заменить
search-replace-all = Заменить всё
search-results =
    { $count ->
        [one] 1 результат
        [few] { $count } результата
        [many] { $count } результатов
       *[other] { $count } результата
    }
search-no-results = Совпадений не найдено

## Редактор
editor-empty-hint = Откройте файл из дерева файлов слева
editor-preview-label = Предпросмотр

# LSP / rust-analyzer
lsp-missing-title = Отсутствует rust-analyzer
lsp-missing-msg = Для работы интеллектуальных функций (автодополнение, ошибки) требуется rust-analyzer. Хотите установить его?
lsp-install-btn = Установить
lsp-installing = Установка rust-analyzer...
lsp-install-success = rust-analyzer успешно установлен. Перезапуск LSP...
lsp-install-error = Ошибка установки: { $error }

## Find References (Shift+F12)
lsp-references-heading = Референсы
lsp-references-searching = Поиск референсов...
lsp-references-none = Референсы не найдены.
lsp-references-found =
    { $count ->
        [one] Найден 1 референс.
        [few] Найдено { $count } референса.
        [many] Найдено { $count } референсов.
       *[other] Найдено { $count } референсов.
    }
lsp-references-error = LSP: Ошибка при поиске референсов.

## Терминал
terminal-unavailable = Терминал недоступен.
terminal-retry = Повторить
terminal-exited = [Процесс завершён — нажмите R для перезапуска]
terminal-close-confirm-title = Закрыть терминал?
terminal-close-confirm-msg = В терминале всё ещё запущен процесс. Вы действительно хотите его завершить?

## Диалог перехода к строке (Ctrl+G)
goto-line-prompt = Перейти к строке:
goto-line-placeholder = номер строки

## Command Palette (Ctrl+Shift+P)
command-palette-heading = Команды
command-palette-placeholder = Поиск команды…
command-palette-no-results = Нет результатов

command-name-open-file = Открыть файл
command-name-project-search = Поиск по проекту
command-name-build = Сборка (Build)
command-name-run = Запуск (Run)
command-name-save = Сохранить текущий файл
command-name-close-tab = Закрыть текущую вкладку
command-name-new-project = Новый проект
command-name-open-project = Открыть проект (в новом окне)
command-name-open-folder = Открыть папку (в этом окне)
command-name-toggle-left = Переключить панель файлов
command-name-toggle-right = Переключить ИИ-панель
command-name-toggle-build = Переключить build-терминал
command-name-toggle-float = Переключить плавающую ИИ-панель
command-name-show-about = О программе
command-name-show-settings = Настройки
command-name-quit = Выйти из PolyCredo Editor
command-name-plugin-hello = Плагин: Поздороваться
command-name-plugin-gemini = Плагин: Спросить Gemini
command-name-show-plugins = Плагины

## Быстрое открытие файла (Ctrl+P)
file-picker-heading = Открыть файл
file-picker-placeholder = Быстрое открытие файла…
file-picker-no-results = Нет результатов
file-picker-count = { $count } файлов
file-picker-count-filtered = { $filtered }/{ $total } файлов
file-picker-more = … и ещё { $count }

## Поиск по проекту (Ctrl+Shift+F)
project-search-heading = Поиск в проекте
project-search-placeholder = Поиск в проекте…
project-search-hint = Поисковый запрос…
project-search-btn = Найти
project-search-loading = Поиск…
project-search-result-label = Результаты для «{ $query }» ({ $count })
project-search-results =
    { $count ->
        [one] 1 результат
        [few] { $count } результата
        [many] { $count } результатов
       *[other] { $count } результата
    }
project-search-no-results = Нет результатов
project-search-max-results = Показано не более { $max } результатов

## Общие кнопки
btn-ok = OK
btn-confirm = Подтвердить
btn-cancel = Отмена
btn-close = Закрыть
btn-browse = Обзор…
btn-create = Создать
btn-open = Открыть
btn-refresh = Обновить
btn-save = Сохранить
btn-rename = Переименовать
btn-copy = Kopировать
btn-paste = Вставить
btn-delete = Удалить
btn-name-label = Имя:

## AI-панель
ai-panel-title = AI-Терминал
ai-tool-not-found = Инструмент { $tool } не найден в PATH.
ai-tool-detecting = Определение AI-инструментов…
ai-label-assistant = Ассистент:
ai-tool-status-checking = { $tool } (проверка…)
ai-tool-status-available = { $tool } (установлен)
ai-tool-status-missing = { $tool } (нет в PATH)
ai-hover-reverify = Повторно проверить доступность AI CLI-инструментов
ai-hover-checking = Проверка доступности AI CLI-инструментов…
ai-hover-start = Запускает { $tool } (`{ $cmd }`) в терминале
ai-hover-missing = Команда `{ $cmd }` не найдена в PATH. Установите инструмент и нажмите ↻.
ai-btn-start = ▶ Запустить
ai-btn-sync = ⟳ Sync
ai-hover-sync = Отправить контекст (открытые файлы, ошибки сборки) ИИ-агенту
ai-diff-heading = Проверка изменений, предложенных ИИ
ai-diff-new-file = Предложен новый файл
ai-plugin-bar-label = ИИ:
ai-plugin-bar-settings = ⚙
ai-plugin-bar-start-hover = Запустить чат с выбранным плагином
ai-plugin-bar-settings-hover = Открыть настройки выбранного плагина
ai-float-dock = Прикрепить к панели
ai-float-undock = Открепить в плавающее окно
ai-viewport-open = Открыть в отдельном окне
ai-tab-close-hover = Закрыть вкладку
ai-tab-new-hover = Новая вкладка терминала
ai-staged-bar-msg = ИИ предложил изменения в проекте
ai-staged-bar-review = Проверить изменения
ai-staged-bar-promote-all = Принять все
ai-staged-modal-hint = Нажмите на файл, чтобы просмотреть различия и принять изменения:
ai-staged-files = Предложенные изменения (Песочница)
ai-staged-new = [НОВЫЙ]
ai-staged-mod = [ИЗМ]
ai-staged-del = [УДАЛЕН]
ai-promotion-success-title = Изменения применены
ai-promotion-success-body = Следующий файл был успешно обновлен в вашем проекте:
ai-promotion-success = Изменения были успешно применены к проекту.
ai-promotion-all-success = Успешно перенесено { $count } файлов в проект.
ai-promotion-failed = Не удалось применить изменения: { $error }

## Синхронизация перед запуском ИИ
ai-sync-title = Синхронизация перед запуском
ai-sync-msg = Обнаружены различия между проектом и песочницей. Последние версии файлов должны быть синхронизированы.
ai-sync-to-sandbox = Обновить песочницу ({ $count } новее в проекте)
ai-sync-to-project = Перенести в проект ({ $count } новее в песочнице)
ai-sync-btn-sync = Синхронизировать и запустить
ai-sync-btn-skip = Запустить без синхронизации

## Разрешения плагинов
plugin-auth-bar-msg = Плагин «{ $name }» запрашивает доступ к интернету ({ $hosts }).
plugin-auth-bar-allow = Разрешить и запустить
plugin-auth-bar-deny = Запретить

## Настройки
settings-title = Настройки
settings-category-general = Общие
settings-category-editor = Редактор
settings-category-ai = ИИ-агенты
settings-language = Язык
settings-language-restart = Язык изменится немедленно.
settings-theme = Тема
settings-theme-dark = Темная
settings-theme-light = Светлая
settings-auto-show-diff = Автоматически открывать предпросмотр изменений ИИ
settings-safe-mode = Безопасный режим (Проект только для чтения)
settings-safe-mode-hint = При включении дерево файлов и билд по умолчанию переходят в Sandbox, а прямое сохранение в проект заблокировано.
settings-diff-mode = Отображение AI Diff
settings-diff-inline = Совмещенное (+ / -)
settings-diff-side-by-side = Рядом
settings-editor-font = Редактор — размер шрифта
settings-ai-font = AI-терминал — размер шрифта
settings-default-path = Путь к проектам по умолчанию
settings-creates-in = Будет создано в:
settings-ai-name = Имя ассистента
settings-ai-command = Команда (бинарный файл)
settings-ai-args = Параметры (необязательно)
settings-ai-add = Добавить агента
settings-ai-hint = Здесь вы можете определить свои собственные инструменты CLI (например, gemini, claude, aider). Если список пуст, будут использоваться настройки по умолчанию.
settings-blacklist = Черный список (запрещенные файлы для плагинов)
settings-blacklist-add = Добавить шаблон
settings-blacklist-hint = Поддерживает шаблоны типа *.env, secret/* или конкретные имена файлов. Автоматически запрещает файлы в .gitignore.

## Плагины
plugins-title = Менеджер плагинов
plugins-list-label = Список плагинов
plugins-no-selection = Выберите плагин из списка слева
plugins-enabled-label = Включить этот плагин
plugins-config-label = Конфигурация плагина:
plugins-unknown-agent = Неизвестный агент
plugins-category-ai = 🤖 ИИ-агенты
plugins-category-general = ⚙ Общее
plugins-item-settings = Настройки
plugins-item-welcome = Обзор
plugins-welcome-title = Добро пожаловать в Менеджер плагинов
plugins-welcome-text = PolyCredo Editor использует современную систему плагинов на базе технологии WebAssembly (WASM). Это обеспечивает высокую производительность и максимальную безопасность — плагины работают в изолированной среде (песочнице) и имеют доступ только к тому, что вы явно разрешите.
plugins-welcome-hint = Выберите категорию или конкретный плагин в списке слева для его настройки.
plugins-security-info = 🛡 Безопасность: Вы можете управлять черным списком файлов/папок в основных настройках.
plugins-settings-saved = Настройки плагинов сохранены. Для некоторых изменений рекомендуется перезапуск.
plugins-placeholder-api-key = API-ключ (например, Gemini, Anthropic)

## Gemini AI

## Семантическая индексация (RAG)
semantic-indexing-title = Семантическая индексация проекта
semantic-indexing-init = Инициализация модели ML (загрузка)...
semantic-indexing-processing = Обработка: { $processed } / { $total }
semantic-indexing-btn-bg = Запустить в фоне
semantic-indexing-status-bar = Индексация проекта...

## Настройки
settings-suggested-patterns = Рекомендуемые шаблоны:

## Ошибка плагина
plugin-error-title = Ошибка плагина
plugin-error-heading = Сбой плагина

## Файлы
file-tree-new-file = Новый файл
file-tree-new-dir = Новая папка
file-tree-rename = Переименовать
file-tree-copy = Копировать
file-tree-paste = Вставить
file-tree-delete = Удалить
file-tree-confirm-delete = Удалить { $name }?
file-tree-unsafe-name = Недопустимое имя: не должно содержать /, \ или ..
file-tree-outside-project = Путь выведет за пределы проекта
file-tree-paste-error = Невозможно вставить: { $reason }
file-tree-create-dir-error = Невозможно создать папку: { $reason }
file-tree-create-file-error = Невозможно создать файл: { $reason }
file-tree-rename-error = Невозможно переименовать: { $reason }
file-tree-delete-error = Невозможно удалить: { $reason }

## Диалог внешнего конфликта
conflict-title = Файл изменён извне
conflict-message = Файл «{ $name }» был изменён (вероятно, при переносе из песочницы), но в редакторе есть несохранённые изменения.
conflict-choose = Выберите, какую версию вы хотите сохранить:
conflict-load-disk = Перезаписать из песочницы
conflict-keep-editor = Сохранить из проекта
conflict-dismiss = Отмена
conflict-hover-disk = Отменить несохранённые изменения в редакторе и загрузить версию, только что перенесённую из песочницы
conflict-hover-keep = Оставить текущие изменения в редакторе; версия из песочницы на диске будет перезаписана при следующем сохранении (Ctrl+S)
conflict-hover-dismiss = Закрыть уведомление без внесения изменений

md-open-external = ↗ Открыть во внешнем браузере

svg-open-external = ↗ Открыть предпросмотр в браузере

svg-modal-title = SVG-файл
svg-modal-body = Этот файл является SVG-изображением. Хотите открыть его во внешнем браузере или редактировать как XML-текст?
svg-modal-edit = Редактировать как текст

## Диалог синхронизации удаления в песочнице
sandbox-delete-title = Файл удален в песочнице
sandbox-delete-msg = Файл «{ $name }» был удален в ИИ-песочнице, но все еще существует в проекте. Что вы хотите сделать?
sandbox-delete-keep-project = Оставить в проекте (восстановить в песочницу)
sandbox-delete-also-project = Удалить также в проекте
ai-chat-title = AI Chat Assistant
ai-chat-label-response = Response:
ai-chat-loading = AI is thinking…
ai-chat-label-prompt = Your prompt:
ai-chat-placeholder-prompt = Enter instructions for AI (e.g. "Explain this code")...
ai-chat-btn-send = Send
ai-chat-btn-new = New Thread
ai-chat-settings-title = AI Settings
ai-chat-label-language = Language:
ai-chat-btn-reset = Reset
ai-chat-label-system-prompt = System Prompt:
ai-chat-default-prompt = Expert Rust Developer.
command-name-plugin-ai-chat = Plugin: Ask AI Agent
command-name-plugin-ollama = Plugin: Ask Ollama
command-name-open-file = Open File
command-name-project-search = Search in Project
command-name-build = Build
command-name-run = Run
command-name-save = Save Current File
command-name-close-tab = Close Current Tab
command-name-new-project = New Project
command-name-open-project = Open Project (in new window)
command-name-open-folder = Open Folder (in this window)
command-name-toggle-left = Toggle File Panel
command-name-toggle-right = Toggle AI Panel
command-name-toggle-build = Toggle Build Terminal
command-name-toggle-float = Toggle Floating AI Panel
command-name-show-about = About
command-name-show-settings = Settings
command-name-quit = Quit PolyCredo Editor
command-name-plugin-hello = Plugin: Say Hello
command-name-plugin-gemini = Plugin: Ask Gemini
plugins-title = Plugin Manager
plugins-list-label = Plugins List
plugins-no-selection = Select a plugin from the list on the left
plugins-enabled-label = Enable this plugin
plugins-config-label = Plugin Configuration:
plugins-unknown-agent = Unknown Agent
plugins-category-ai = 🤖 AI Agents
plugins-category-general = ⚙ General
plugins-item-settings = Settings
plugins-item-welcome = Overview
plugins-welcome-title = Welcome to Plugin Manager
plugins-welcome-text = PolyCredo Editor utilizes a modern plugin system based on WebAssembly (WASM). This ensures high performance and maximum security — plugins run in an isolated environment (sandbox) and only have access to what you explicitly authorize.
plugins-welcome-hint = Select a category or a specific plugin from the list on the left to configure it.
plugins-security-info = 🛡 Security: You can manage the file/directory blacklist in the main Settings.
plugins-settings-saved = Plugin settings saved. Restart recommended for some changes.
plugins-placeholder-api-key = API Key (e.g. Gemini, Anthropic)
plugins-placeholder-model = Model ID (e.g. gemini-1.5-flash)
command-name-show-plugins = Plugins
ai-chat-title = AI Chat Assistant
ai-chat-label-response = Response:
ai-chat-loading = AI is thinking…
ai-chat-label-prompt = Your prompt:
ai-chat-placeholder-prompt = Enter instructions for AI (e.g. "Explain this code")...
ai-chat-btn-send = Send
ai-chat-btn-new = New Thread
ai-chat-settings-title = AI Settings
ai-chat-label-language = Language:
ai-chat-btn-reset = Reset
ai-chat-label-system-prompt = System Prompt:
ai-chat-default-prompt = Expert Rust Developer.
command-name-plugin-gemini = Plugin: Ask Gemini
command-name-plugin-ollama = Plugin: Ask Ollama
command-name-plugin-ai-chat = Plugin: Ask AI Agent

## Support Modal
support-modal-title = Поддержать разработку PolyCredo
support-modal-body = PolyCredo Editor разрабатывается с акцентом на приватность, скорость и безопасную интеграцию ИИ-ассистентов. Если вам нравится проект, мы будем благодарны за любую поддержку. Ваши пожертвования помогают нам уделять больше времени разработке новых функций и поддержке.
support-modal-github = Подписаться на GitHub
support-modal-donate = Поддержать разработку
semantic-indexing-btn-stop = Остановить индексацию

menu-build-all = Все пакеты
build-all-status-running = Сборка выполняется…
build-all-status-ok = ✔ Все пакеты успешно собраны
build-all-status-error = ✘ Сборка завершилась с ошибкой (код { $code })
build-all-status-waiting = Ожидание запуска…
build-all-waiting-output = Запуск scripts/build-all.sh…
build-all-btn-close = Закрыть
