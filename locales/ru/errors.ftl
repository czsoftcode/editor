# Сообщения об ошибках и информационные сообщения

## Файлы
error-file-read = Ошибка чтения файла: { $path }
error-file-write = Ошибка записи файла: { $path }
error-file-save = Ошибка сохранения «{ $name }»: { $reason }
error-file-deleted = Файл был удалён: { $path }
error-file-delete = Ошибка удаления { $name }: { $reason }
error-file-rename = Ошибка переименования: { $reason }
error-file-create = Ошибка создания файла { $name }: { $reason }
error-file-read-only-error = Не удалось сохранить «{ $name }», так как файл не был корректно прочитан. Эта вкладка теперь доступна только для чтения во избежание потери данных.
error-safe-mode-blocked = Проект находится в безопасном режиме (только для чтения). Вы можете вносить изменения только в Sandbox или отключите безопасный режим в Настройках.
error-file-watch = Ошибка отслеживания файлов

## Папки
error-folder-create = Ошибка создания папки { $name }: { $reason }
error-folder-delete = Ошибка удаления папки { $name }: { $reason }

## Проекты
error-project-create = Ошибка создания проекта: { $reason }
error-project-open = Ошибка открытия проекта: { $path }
error-project-not-found = Проект не найден: { $path }
error-project-dir-create = Невозможно создать каталог проекта: { $reason }
error-cmd-failed = Команда завершилась с кодом: { $code }
error-cmd-start = Не удалось запустить команду: { $reason }
error-projects-dir-prepare = Невозможно подготовить каталог проектов: { $reason }

## Сессия
error-session-restore = Проект из предыдущей сессии не найден: { $path }
error-session-load = Ошибка загрузки сессии.
error-session-save = Ошибка сохранения сессии.

## Сборка
error-build-parse = Ошибка разбора вывода сборки.

## Буфер обмена
error-clipboard = Ошибка буфера обмена: { $reason }

## IPC
error-ipc-connect = Ошибка подключения к запущенному экземпляру.

## Общие
error-unknown = Произошла неизвестная ошибка.

## Информационные сообщения (toast info)
info-file-saved = Файл сохранён.
info-project-created = Проект { $name } успешно создан.
info-session-restored =
    { $count ->
        [one] Восстановлено 1 окно из предыдущей сессии.
        [few] Восстановлено { $count } окна из предыдущей сессии.
        [many] Восстановлено { $count } окон из предыдущей сессии.
       *[other] Восстановлено { $count } окна из предыдущей сессии.
    }
