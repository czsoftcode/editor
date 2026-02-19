/// Bezpečný název souboru nebo adresáře: bez path separátorů,
/// traversal komponent (`.` / `..`) a null bytů.
pub(crate) fn is_safe_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

/// Platný název projektu: povoleny pouze ASCII alfanumerika, podtržítko
/// a pomlčka. Název nesmí být prázdný ani začínat pomlčkou.
pub(crate) fn is_valid_project_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('-') {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Bezpečný název projektu = musí splňovat obě pravidla najednou.
/// Použít všude, kde se název stane názvem adresáře na disku.
#[allow(dead_code)]
pub(crate) fn is_safe_project_name(name: &str) -> bool {
    is_valid_project_name(name) && is_safe_filename(name)
}
