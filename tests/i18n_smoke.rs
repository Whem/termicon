use termicon_core::i18n::{set_locale, t, Locale};

#[test]
fn i18n_menu_keys_resolve() {
    set_locale(Locale::English);
    assert_eq!(t("menu.file"), "File");
    assert_eq!(t("menu.settings"), "Settings...");

    set_locale(Locale::Hungarian);
    assert_eq!(t("menu.file"), "Fájl");
    assert_eq!(t("menu.settings"), "Beállítások...");
}


