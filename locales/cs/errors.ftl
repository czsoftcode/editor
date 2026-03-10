# Chybové a informační hlášky

## Soubory
error-file-read = Chyba čtení souboru: { $path }
error-file-write = Chyba zápisu souboru: { $path }
error-file-save = Chyba ukládání „{ $name }": { $reason }
error-file-deleted = Soubor byl smazán: { $path }
error-file-delete = Chyba mazání souboru { $name }: { $reason }
error-file-rename = Chyba přejmenování: { $reason }
error-file-create = Chyba vytváření souboru { $name }: { $reason }
error-file-read-only-error = Nelze uložit soubor „{ $name }“, protože jej nebylo možné správně přečíst. Tento tab je nyní pouze pro čtení, aby se zabránilo ztrátě dat.
error-file-watch = Chyba sledování souborů

## Složky
error-folder-create = Chyba vytváření složky { $name }: { $reason }
error-folder-delete = Chyba mazání složky { $name }: { $reason }

## Projekty
error-project-create = Chyba vytváření projektu: { $reason }
error-project-open = Chyba otevření projektu: { $path }
error-project-not-found = Projekt nebyl nalezen: { $path }
error-project-dir-create = Nelze vytvořit adresář projektu: { $reason }
error-cmd-failed = Příkaz selhal s kódem: { $code }
error-cmd-start = Nepodařilo se spustit příkaz: { $reason }
error-projects-dir-prepare = Nelze připravit adresář projektů: { $reason }

## Session
error-session-restore = Projekt z předchozí session nebyl nalezen: { $path }
error-session-load = Chyba načítání session.
error-session-save = Chyba ukládání session.

## Build
error-build-parse = Chyba parsování výstupu buildu.

## Clipboard
error-clipboard = Chyba přístupu ke schránce: { $reason }

## IPC
error-ipc-connect = Chyba připojení k běžící instanci.

## Obecné
error-unknown = Nastala neznámá chyba.

## Informační hlášky (toast info)
info-file-saved = Soubor uložen.
info-file-already-saved = Soubor už je uložen.
info-project-created = Projekt { $name } byl vytvořen.
info-session-restored =
    { $count ->
        [one] Obnoveno 1 okno z předchozí session.
       *[other] Obnoveno { $count } oken z předchozí session.
    }

## Strážce neuložených změn
unsaved_close_guard_save_failed = Nepodařilo se uložit „{ $name }“ během zavírání: { $reason }
