use std::sync::LazyLock;

use icu_collator::{Collator, CollatorOptions};
use icu_provider::DataLocale;
use log::{error, info, warn};

/// Gets the browser data locale. The locale is cached for the lifetime of the application, so it
/// requires a page reload to pick up locale changes.
pub fn get_locale() -> &'static DataLocale {
    fn try_parse_browser_locale() -> DataLocale {
        let navigator = match web_sys::window().map(|win| win.navigator()) {
            Some(navigator) if !(navigator.is_null() || navigator.is_undefined()) => navigator,
            Some(_) => {
                error!("Navigator was null or undefined, using default locale");
                return Default::default();
            }
            None => {
                error!("Unable to find the global window, using default locale");
                return Default::default();
            }
        };
        let languages = navigator.languages();
        // I have no idea how much validation wasm_bindgen/web_sys does so I have no idea if the
        // array can ever actually be null/undefined or if wasm_bindgen will throw in that case.
        if !(languages.is_null() || languages.is_undefined()) {
            for (i, lang) in languages.iter().enumerate() {
                if let Some(lang) = lang.as_string() {
                    match lang.parse() {
                        Ok(locale) => return locale,
                        Err(e) => {
                            warn!("Unable to parse locale {lang} at index {i}: {e}");
                        }
                    }
                } else {
                    warn!("Locale entry at index {i} is not a string");
                }
            }
        }

        match navigator.language() {
            Some(lang) => match lang.parse() {
                Ok(locale) => return locale,
                Err(e) => {
                    warn!("Unable to parse local {lang} from the navigator.language: {e}");
                }
            },
            None => {
                warn!("navigator.language was missing");
            }
        }

        info!("Could not find or parse a locale, falling back to the default locale.");

        Default::default()
    }

    static LOCALE: LazyLock<DataLocale> = LazyLock::new(try_parse_browser_locale);
    &LOCALE
}

/// Gets a collator for the the current locale, or the default locale if the current locale is
/// unavailable.
pub fn get_collator() -> Collator {
    let locale = get_locale();
    match Collator::try_new(locale, CollatorOptions::new()) {
        Ok(collator) => collator,
        Err(e) => {
            warn!("Unable to create collator with the current locale {locale}: {e}",);
            Collator::try_new(&Default::default(), CollatorOptions::new())
                .expect("Unable to create a collator with the default settings")
        }
    }
}
