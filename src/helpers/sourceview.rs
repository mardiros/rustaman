use relm4::adw;
use sourceview5::{self, prelude::*};

pub fn create_buffer(language: &str) -> sourceview5::Buffer {
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);

    let langmngr = sourceview5::LanguageManager::default();
    let stmngr = sourceview5::StyleSchemeManager::default();
    let sm = adw::StyleManager::default();

    if let Some(ref language) = langmngr.language(language) {
        buffer.set_language(Some(language));
    } else {
        error!(
            "Can't find {}.lang lang in {:?}",
            language,
            langmngr.search_path()
        )
    }
    let scheme_id = match sm.color_scheme() {
        adw::ColorScheme::ForceLight => "rustaman-light".to_owned(),
        _ => "rustaman-dark".to_owned(),
    };

    if let Some(ref scheme) = stmngr.scheme(scheme_id.as_str()) {
        buffer.set_style_scheme(Some(scheme));
    } else {
        error!(
            "Can't find {}.xml theme in {:?}",
            scheme_id,
            stmngr.search_path()
        )
    }
    buffer
}
