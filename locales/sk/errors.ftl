# Chybové a informačné správy

## Súbory
error-file-read = Chyba čítania súboru: { $path }
error-file-write = Chyba zápisu súboru: { $path }
error-file-save = Chyba ukladania „{ $name }": { $reason }
error-file-deleted = Súbor bol odstránený: { $path }
error-file-delete = Chyba pri odstraňovaní { $name }: { $reason }
error-file-rename = Chyba premenovania: { $reason }
error-file-create = Chyba pri vytváraní súboru { $name }: { $reason }
error-file-read-only-error = Súbor „{ $name }“ nie je možné uložiť, pretože sa ho nepodarilo správne prečítať. Táto karta je teraz len na čítanie, aby sa zabránilo strate údajov.
error-file-watch = Chyba sledovania súborov

## Priečinky
error-folder-create = Chyba pri vytváraní priečinka { $name }: { $reason }
error-folder-delete = Chyba pri odstraňovaní priečinka { $name }: { $reason }

## Projekty
error-project-create = Chyba pri vytváraní projektu: { $reason }
error-project-open = Chyba pri otváraní projektu: { $path }
error-project-not-found = Projekt nenájdený: { $path }
error-project-dir-create = Nemožno vytvoriť adresár projektu: { $reason }
error-cmd-failed = Príkaz zlyhal s kódom: { $code }
error-cmd-start = Nepodarilo sa spustiť príkaz: { $reason }
error-projects-dir-prepare = Nemožno pripraviť adresár projektov: { $reason }

## Relácia
error-session-restore = Projekt z predchádzajúcej relácie nenájdený: { $path }
error-session-load = Chyba načítania relácie.
error-session-save = Chyba ukladania relácie.

## Build
error-build-parse = Chyba spracovania výstupu buildu.

## Schránka
error-clipboard = Chyba schránky: { $reason }

## IPC
error-ipc-connect = Chyba pripojenia k bežiacej inštancii.

## Všeobecné
error-unknown = Vyskytla sa neznáma chyba.

## Informačné správy (toast info)
info-file-saved = Súbor uložený.
info-file-already-saved = Súbor je už uložený.
info-project-created = Projekt { $name } bol úspešne vytvorený.
info-session-restored =
    { $count ->
        [one] Obnovené 1 okno z predchádzajúcej relácie.
        [few] Obnovené { $count } okná z predchádzajúcej relácie.
       *[other] Obnovených { $count } okien z predchádzajúcej relácie.
    }

## Strážca neuložených zmien
unsaved_close_guard_save_failed = Nepodarilo sa uložiť „{ $name }“ počas zatvárania: { $reason }
