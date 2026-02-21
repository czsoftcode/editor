# Общие элементы интерфейса пользователя

## Панели
panel-files = Файлы
panel-build = Build
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
btn-create-deb = Создать .deb

## Строка состояния
statusbar-line-col = Строка { $line }, Столбец { $col }
statusbar-encoding = UTF-8
statusbar-unsaved = Не сохранено
statusbar-saving = Сохранение…
statusbar-saved = Сохранено
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

## Настройки
settings-title = Настройки
settings-language = Язык
settings-language-restart = Изменение языка применяется немедленно.
settings-theme = Тема
settings-theme-dark = Тёмная
settings-theme-light = Светлая
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
conflict-message = Файл «{ $name }» был изменён другой программой, но в редакторе есть несохранённые изменения.
conflict-choose = Выберите, какую версию оставить:
conflict-load-disk = Загрузить с диска
conflict-keep-editor = Оставить мои изменения
conflict-dismiss = Закрыть
conflict-hover-disk = Отменить изменения редактора и загрузить версию с диска
conflict-hover-keep = Сохранить изменения редактора; файл на диске будет перезаписан при сохранении
conflict-hover-dismiss = Закрыть уведомление без изменений

md-open-external = ⧉ Открыть во внешнем просмотрщике

svg-open-external = ⧉ Открыть предпросмотр во внешнем просмотрщике

svg-modal-title = SVG-файл
svg-modal-body = Этот файл является SVG-изображением. Открыть его во внешнем просмотрщике или редактировать как XML-текст?
svg-modal-edit = Редактировать как текст
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
