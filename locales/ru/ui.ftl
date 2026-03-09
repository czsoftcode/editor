# Общие элементы интерфейса пользователя

## Панели
panel-files = Файлы
btn-tree-project = Проект
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
btn-run = ▶ Run
btn-run-new = ▶ Run+
btn-test = ▶ Test
btn-clean = ✖ Clean
btn-create-deb = Create .deb
hover-create-deb = Собрать и создать разработческий .deb-пакет с номером сборки
btn-run-profile = ▶ Запустить...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Редактировать
runner-none = Профили не определены.
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
cancel-confirm-title = Отменить изменения?
cancel-confirm-msg = Вы уверены, что хотите отменить все несохраненные изменения и закрыть это окно?
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
ai-diff-heading = Проверка изменений, предложенных ИИ
ai-diff-new-file = Предложен новый файл
ai-float-dock = Прикрепить к панели
ai-float-undock = Открепить в плавающее окно
ai-viewport-open = Открыть в отдельном окне
ai-tab-close-hover = Закрыть вкладку
ai-tab-new-hover = Новая вкладка терминала
ai-staged-bar-msg = ИИ предложил изменения в проекте
ai-staged-bar-review = Проверить изменения
ai-staged-bar-promote-all = Принять все
ai-staged-modal-hint = Нажмите на файл, чтобы просмотреть различия и принять изменения:
ai-staged-files = Предложенные изменения
ai-staged-new = [НОВЫЙ]
ai-staged-mod = [ИЗМ]
ai-staged-del = [УДАЛЕН]
ai-promotion-success-title = Изменения применены
ai-promotion-success-body = Следующий файл был успешно обновлен в вашем проекте:
ai-promotion-success = Изменения были успешно применены к проекту.
ai-promotion-all-success = Успешно перенесено { $count } файлов в проект.
ai-promotion-failed = Не удалось применить изменения: { $error }

## Синхронизация перед запуском ИИ

## Разрешения плагинов

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
settings-light-variant = Вариант светлой темы
settings-light-variant-warm-ivory = Теплая слоновая кость
settings-light-variant-cool-gray = Холодный серый
settings-light-variant-sepia = Сепия
settings-auto-show-diff = Автоматически открывать предпросмотр изменений ИИ
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
conflict-message = Файл «{ $name }» был изменён вне редактора, но в редакторе есть несохранённые изменения.
conflict-choose = Выберите, какую версию вы хотите сохранить:
conflict-load-disk = Загрузить с диска
conflict-keep-editor = Сохранить версию редактора
conflict-dismiss = Отмена
conflict-hover-disk = Отменить несохранённые изменения в редакторе и загрузить версию, изменённую на диске
conflict-hover-keep = Оставить текущие изменения в редакторе; версия на диске будет перезаписана при следующем сохранении (Ctrl+S)
conflict-hover-dismiss = Закрыть уведомление без внесения изменений

md-open-external = ↗ Открыть во внешнем браузере
md-layout-pod-sebou = Друг под другом
md-layout-vedle-sebe = Рядом
md-layout-jenom-kod = Только код
md-layout-jenom-nahled = Только предпросмотр

svg-open-external = ↗ Открыть предпросмотр в браузере

svg-modal-title = SVG-файл
svg-modal-body = Этот файл является SVG-изображением. Хотите открыть его во внешнем браузере или редактировать как XML-текст?
svg-modal-edit = Редактировать как текст

settings-conflict-title = Настройки изменены
settings-conflict-message = Настройки были обновлены в другом окне. Перезагрузить или продолжить редактирование текущего черновика?
settings-conflict-reload = Перезагрузить
settings-conflict-keep = Продолжить редактирование

## Command Palette – ИИ-плагины

## Support Modal
support-modal-title = Поддержать разработку PolyCredo
support-modal-body = PolyCredo Editor разрабатывается с акцентом на приватность, скорость и безопасную интеграцию ИИ-ассистентов. Если вам нравится проект, мы будем благодарны за любую поддержку. Ваши пожертвования помогают нам уделять больше времени разработке новых функций и поддержке.
support-modal-github = Подписаться на GitHub
support-modal-donate = Поддержать разработку
semantic-indexing-btn-stop = Остановить индексацию

dep-wizard-appimagetool-desc = ...
