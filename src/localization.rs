// SPDX-License-Identifier: GPL-3.0-only

use i18n_embed::{
    DesktopLanguageRequester, LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

/// Selects the translations matching the system language, falling back to English
pub fn init() {
    let requested = DesktopLanguageRequester::requested_languages();
    if let Err(err) = i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &requested) {
        println!("failed to load translations: {err}");
    }
    // Unicode isolation marks around arguments would show up as boxes in the panel
    LANGUAGE_LOADER.set_use_isolating(false);
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localization::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localization::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}
