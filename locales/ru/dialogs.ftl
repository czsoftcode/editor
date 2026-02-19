# Диалоги приложения

## Стартовый диалог
startup-title = PolyCredo Editor
startup-subtitle = AI Polyglot Code Editor
startup-open-folder = Открыть папку
startup-new-project = Новый проект
startup-recent-projects = Недавние проекты
startup-no-recent = Нет недавних проектов
startup-quit = Выйти
startup-missing-session =
    { $count ->
        [one] 1 проект из предыдущей сессии не найден.
        [few] { $count } проекта из предыдущей сессии не найдены.
        [many] { $count } проектов из предыдущей сессии не найдены.
       *[other] { $count } проекта из предыдущей сессии не найдены.
    }
startup-missing-session-label = Проекты из предыдущей сессии не удалось восстановить:
startup-path-label = Путь:

## Диалог открытия проекта
open-project-title = Открыть проект
open-project-question = Проект уже открыт. Где открыть новый?
open-project-in-window = В этом окне
open-project-new-window = В новом окне
open-project-cancel = Отмена

## Мастер нового проекта
wizard-title = Новый проект
wizard-project-type = Тип проекта
wizard-project-name = Название проекта
wizard-project-path = Путь
wizard-type-rust = Rust
wizard-type-symfony = Symfony
wizard-creating = Создание проекта…
wizard-name-hint = Допускаются только буквы, цифры, _ и -
wizard-error-empty-name = Название проекта не может быть пустым.
wizard-error-invalid-name = Недопустимое название. Допускаются только буквы, цифры, _ и -.
wizard-error-starts-with-dash = Название не должно начинаться с дефиса.
wizard-error-exists = Проект с таким названием уже существует по указанному пути.
wizard-error-create = Ошибка создания проекта: { $reason }

## Диалог закрытия проекта
close-project-title = Закрыть проект
close-project-message = Вы уверены, что хотите закрыть этот проект?
close-project-confirm = Закрыть
close-project-cancel = Отмена

## Диалог выхода
quit-title = Выход из приложения
quit-message = Вы уверены, что хотите выйти из PolyCredo Editor?
quit-confirm = Выйти
quit-cancel = Отмена

## Диалог О программе
about-title = О программе
about-version = Версия { $version }
about-build = Build { $build }
about-description = AI Polyglot Code Editor
about-copyright = © 2024–2026 PolyCredo
about-close = Закрыть

## Диалоги подтверждения (общие)
confirm-delete-file = Вы уверены, что хотите удалить { $name }?
confirm-delete-folder = Вы уверены, что хотите удалить { $name } и всё его содержимое?
confirm-delete-confirm = Удалить
confirm-delete-cancel = Отмена

## Переименование
rename-title = Переименовать
rename-label = Новое имя:
rename-confirm = Переименовать
rename-cancel = Отмена
