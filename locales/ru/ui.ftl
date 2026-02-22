# Общие элементы интерфейса пользователя

## Панели
panel-files = Файлы
panel-files-sandbox = Файлы (Sandbox)
btn-tree-project = Проект
btn-tree-sandbox = Sandbox
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
btn-create-deb = Создать .deb
hover-create-deb-disabled = Невозможно создать пакет в режиме песочницы. Переключитесь на Sandbox ВЫКЛ.
btn-run-profile = ▶ Запустить профиль...
btn-git-profile =  Git...
btn-edit-profiles = ⚙ Изменить
runner-none = Профили не определены.

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
hover-git-disabled-sandbox = Git operations are disabled until all sandbox changes are resolved (use 'Review Changes' or 'Promote All' in the yellow bar).

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
btn-browse = Обзор…
btn-create = Создать
btn-open = Открыть
btn-refresh = Обновить
btn-save = Сохранить
btn-rename = Переименовать
btn-copy = Копировать
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
ai-float-dock = Прикрепить к панели
ai-float-undock = Открепить в плавающее окно
ai-viewport-open = Открыть в отдельном окне
ai-tab-close-hover = Закрыть вкладку
ai-tab-new-hover = Новая вкладка терминала
ai-staged-bar-msg = ИИ предложил изменения в проекте
ai-staged-bar-review = Проверить изменения
ai-staged-bar-promote-all = Принять всё
ai-staged-modal-hint = Нажмите на файл, чтобы увидеть различия и одобрить изменения:
ai-staged-files = Предложенные изменения (Sandbox)
ai-staged-new = [НОВЫЙ]
ai-staged-mod = [MOD]
ai-staged-del = [УДАЛЁН]
ai-promotion-success-title = Изменения применены
ai-promotion-success-body = Следующий файл успешно обновлён в вашем проекте:
ai-promotion-success = Изменения успешно применены к проекту.
ai-promotion-failed = Не удалось применить изменения: { $error }

## Настройки
settings-title = Настройки
settings-language = Язык
settings-language-restart = Изменение языка применяется немедленно.
settings-theme = Тема
settings-theme-dark = Тёмная
settings-theme-light = Светлая
settings-auto-show-diff = Автоматически открывать просмотр изменений ИИ
settings-diff-mode = Вид ИИ Diff
settings-diff-inline = Вместе (+ / -)
settings-diff-side-by-side = Рядом
settings-editor-font = Редактор — размер шрифта
settings-ai-font = AI-Терминал — размер шрифта
settings-default-path = Путь проектов по умолчанию
settings-creates-in = Будет создано в:

## Дерево файлов
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

md-open-external = ⧉ Открыть во внешнем просмотрщике

svg-open-external = ⧉ Открыть предпросмотр во внешнем просмотрщике

svg-modal-title = SVG-файл
svg-modal-body = Этот файл является SVG-изображением. Открыть его во внешнем просмотрщике или редактировать как XML-текст?
svg-modal-edit = Редактировать как текст

## Диалог синхронизации удаления в песочнице
sandbox-delete-title = Файл удалён в песочнице
sandbox-delete-msg = Файл «{ $name }» был удалён в ИИ-песочнице, но всё ещё существует в проекте. Что вы хотите сделать?
sandbox-delete-keep-project = Оставить в проекте (восстановить в песочницу)
sandbox-delete-also-project = Удалить также в проекте
panel-runners = Runners
btn-run-profile = Run Profile...
btn-edit-profiles = Edit
runner-none = None

## Find References (Shift+F12)
lsp-references-heading = Ссылки
lsp-references-searching = Поиск ссылок...
lsp-references-none = Ссылки не найдены.
lsp-references-found =
    { $count ->
        [one] Найдена 1 ссылка.
        [few] Найдено { $count } ссылки.
       *[other] Найдено { $count } ссылок.
    }
lsp-references-error = LSP: Ошибка при поиске ссылок.

ai-btn-sync = ⟳ Синхр.
ai-hover-sync = Отправить контекст (открытые файлы, ошибки сборки) ИИ-агенту
ai-diff-heading = Проверка предложенных ИИ изменений
ai-diff-new-file = Предложен новый файл
