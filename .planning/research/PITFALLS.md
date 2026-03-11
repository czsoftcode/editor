# Milestone v1.3.0 Research - Pitfalls

## Main risks
- Skryte vazby na `app::cli::*` v settings/state testech.
- Rozbiti approval flow pri presunu `ToolExecutor` a souvisejicich typu.
- Dead imports/mod.rs exporty po odstraneni adresare.
- Regrese v i18n klicich a textovych stavech AI panelu.

## Prevention
- Delat migration po fazich s kazdou fazi overenou `cargo check` a `./check.sh`.
- Pred mazanim modulu vyhledat vsechny `use crate::app::cli` reference (`rg`).
- Udrzet API kontrakt mezi UI a backend casti co nejvic stabilni.
- Prubezne doplnovat regression testy tam, kde meni importovani/stavy.
