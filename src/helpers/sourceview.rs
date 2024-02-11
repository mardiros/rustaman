use sourceview5::{self, prelude::*};

pub fn create_buffer(language: &str) -> sourceview5::Buffer {
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);

    let langmngr = sourceview5::LanguageManager::default();
    let stmngr = sourceview5::StyleSchemeManager::default();

    if let Some(ref language) = langmngr.language(language) {
        buffer.set_language(Some(language));
    } else {
        error!(
            "Can't find {}.lang lang in {:?}",
            language,
            langmngr.search_path()
        )
    }
    if let Some(ref scheme) = stmngr.scheme("rustaman-dark") {
        buffer.set_style_scheme(Some(scheme));
    } else {
        error!(
            "Can't find rustaman-dark.xml theme in {:?}",
            stmngr.search_path()
        )
    }
    buffer
}
