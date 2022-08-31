//! Static files bundled with documentation output.
//!
//! All the static files are included here for centralized access in case anything other than the
//! HTML rendering code (say, the theme checker) needs to access one of these files.
//!
//! Note about types: CSS and JavaScript files are included as `&'static str` to allow for the
//! minifier to run on them. All other files are included as `&'static [u8]` so they can be
//! directly written to a `Write` handle.

use crate::config::EmitType;
use rustc_data_structures::fx::FxHasher;
use std::hash::Hasher;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub(crate) struct StaticFile {
    versioning: Versioning,
    pub(crate) filename: &'static str,
    pub(crate) bytes: &'static [u8],
}

pub(crate) enum Versioning {
    /// This file will never change, no matter what toolchain is used to build it.
    ///
    /// It does not have a resource suffix.
    Unversioned,
    /// This file may change depending on the toolchain.
    ///
    /// It has a resource suffix.
    ToolchainSpecific,
}

impl StaticFile {
    pub(crate) fn as_str(&self) -> &'static str {
        std::str::from_utf8(self.bytes).unwrap()
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.bytes
    }

    pub(crate) fn extension(&self) -> Option<&OsStr> {
        Path::new(self.filename.into()).extension()
    }

    pub(crate) fn should_emit(&self, emit: &[EmitType]) -> bool {
        if emit.is_empty() {
            return true;
        }
        let kind = match self.versioning {
            Versioning::Unversioned => EmitType::Unversioned,
            Versioning::ToolchainSpecific => EmitType::Toolchain,
        };
        emit.contains(&kind)
    }

    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        hasher.write(self.bytes);
        hasher.finish()
    }

    fn suffix(&self, resource_suffix: &str) -> String {
        if resource_suffix == "" {
            format!("-{:016x}", self.hash())
        } else {
            format!("-{:016x}-{}", self.hash(), resource_suffix)
        }
    }

    pub(crate) fn output_filename(&self, dst: &PathBuf, suffix: &str) -> PathBuf {
        let filename = self.filename.rsplit("/").next().unwrap();
        match self.versioning {
            Versioning::Unversioned => dst.join("static").join(filename),
            Versioning::ToolchainSpecific => {
                let suffix_with_hash = self.suffix(suffix);
                Path::new("static").join(suffix_path(filename, &suffix_with_hash))
            }
        }
    }
}

pub(crate) fn suffix_path(filename: &str, resource_suffix: &str) -> PathBuf {
    // We use splitn vs Path::extension here because we might get a filename
    // like `style.min.css` and we want to process that into
    // `style-suffix.min.css`.  Path::extension would just return `css`
    // which would result in `style.min-suffix.css` which isn't what we
    // want.
    let (base, ext) = filename.split_once('.').unwrap();
    let filename = format!("{}{}.{}", base, resource_suffix, ext);
    filename.into()
}

macro_rules! toolchain_specific {
    ($name:ident, $file:literal) => {
        pub(crate) static $name: crate::html::static_files::StaticFile =
            crate::html::static_files::StaticFile {
                versioning: crate::html::static_files::Versioning::ToolchainSpecific,
                filename: $file,
                bytes: include_bytes!($file),
            };
    };
}

macro_rules! unversioned {
    ($name:ident, $file:literal) => {
        pub(crate) static $name: crate::html::static_files::StaticFile =
            crate::html::static_files::StaticFile {
                versioning: crate::html::static_files::Versioning::Unversioned,
                filename: $file,
                bytes: include_bytes!($file),
            };
    };
}

pub(crate) static FILES_TOOLCHAIN: [&'static StaticFile; 21] = [
    &RUSTDOC_CSS,
    &SETTINGS_CSS,
    &NOSCRIPT_CSS,
    &NORMALIZE_CSS,
    &MAIN_JS,
    &SEARCH_JS,
    &SETTINGS_JS,
    &STORAGE_JS,
    &SCRAPE_EXAMPLES_JS,
    &WHEEL_SVG,
    &CLIPBOARD_SVG,
    &DOWN_ARROW_SVG,
    &TOGGLE_MINUS_PNG,
    &TOGGLE_PLUS_PNG,
    &RUST_LOGO_SVG,
    &RUST_FAVICON_SVG,
    &RUST_FAVICON_PNG_16,
    &RUST_FAVICON_PNG_32,
    &themes::DARK,
    &themes::LIGHT,
    &themes::AYU,
];

pub(crate) static FILES_UNVERSIONED: [&'static StaticFile; 16] = [
    &fira_sans::REGULAR,
    &fira_sans::MEDIUM,
    &fira_sans::LICENSE,
    &source_serif_4::REGULAR,
    &source_serif_4::BOLD,
    &source_serif_4::ITALIC,
    &source_serif_4::LICENSE,
    &source_code_pro::REGULAR,
    &source_code_pro::SEMIBOLD,
    &source_code_pro::ITALIC,
    &source_code_pro::LICENSE,
    &nanum_barun_gothic::REGULAR,
    &nanum_barun_gothic::LICENSE,
    &LICENSE_MIT,
    &LICENSE_APACHE,
    &COPYRIGHT,
];

// The file contents of the main `rustdoc.css` file, responsible for the core layout of the page.
toolchain_specific!(RUSTDOC_CSS, "static/css/rustdoc.css");

// The file contents of `settings.css`, responsible for the items on the settings page.
toolchain_specific!(SETTINGS_CSS, "static/css/settings.css");

// The file contents of the `noscript.css` file, used in case JS isn't supported or is disabled.
toolchain_specific!(NOSCRIPT_CSS, "static/css/noscript.css");

// The file contents of `normalize.css`, included to even out standard elements between browser
// implementations.
toolchain_specific!(NORMALIZE_CSS, "static/css/normalize.css");

// The file contents of `main.js`, which contains the core JavaScript used on documentation pages,
// including search behavior and docblock folding, among others.
toolchain_specific!(MAIN_JS, "static/js/main.js");

// The file contents of `search.js`, which contains the search behavior.
toolchain_specific!(SEARCH_JS, "static/js/search.js");

// The file contents of `settings.js`, which contains the JavaScript used to handle the settings
// page.
toolchain_specific!(SETTINGS_JS, "static/js/settings.js");

// The file contents of `storage.js`, which contains functionality related to browser Local
// Storage, used to store documentation settings.
toolchain_specific!(STORAGE_JS, "static/js/storage.js");

// The file contents of `scraped-examples.js`, which contains functionality related to the
// --scrape-examples flag that inserts automatically-found examples of usages of items.
toolchain_specific!(SCRAPE_EXAMPLES_JS, "static/js/scrape-examples.js");

// TODO: figure out where to put this
toolchain_specific!(SCRAPE_EXAMPLES_HELP_MD, "static/scrape-examples-help.md");

// The file contents of `wheel.svg`, the icon used for the settings button.
toolchain_specific!(WHEEL_SVG, "static/images/wheel.svg");

// The file contents of `clipboard.svg`, the icon used for the "copy path" button.
toolchain_specific!(CLIPBOARD_SVG, "static/images/clipboard.svg");

// The file contents of `down-arrow.svg`, the icon used for the crate choice combobox.
toolchain_specific!(DOWN_ARROW_SVG, "static/images/down-arrow.svg");

// The file contents of `toggle-minus.svg`, the icon used for opened toggles.
toolchain_specific!(TOGGLE_MINUS_PNG, "static/images/toggle-minus.svg");

// The file contents of `toggle-plus.svg`, the icon used for closed toggles.
toolchain_specific!(TOGGLE_PLUS_PNG, "static/images/toggle-plus.svg");

// The contents of `COPYRIGHT.txt`, the license listing for files distributed with documentation
// output.
unversioned!(COPYRIGHT, "static/COPYRIGHT.txt");

// The contents of `LICENSE-APACHE.txt`, the text of the Apache License, version 2.0.
unversioned!(LICENSE_APACHE, "static/LICENSE-APACHE.txt");

// The contents of `LICENSE-MIT.txt`, the text of the MIT License.
unversioned!(LICENSE_MIT, "static/LICENSE-MIT.txt");

// The contents of `rust-logo.svg`, the default icon of the documentation.
toolchain_specific!(RUST_LOGO_SVG, "static/images/rust-logo.svg");

// The default documentation favicons (SVG and PNG fallbacks)
toolchain_specific!(RUST_FAVICON_SVG, "static/images/favicon.svg");
toolchain_specific!(RUST_FAVICON_PNG_16, "static/images/favicon-16x16.png");
toolchain_specific!(RUST_FAVICON_PNG_32, "static/images/favicon-32x32.png");

/// The built-in themes given to every documentation site.
pub(crate) mod themes {
    // The "light" theme, selected by default when no setting is available. Used as the basis for
    // the `--check-theme` functionality.
    toolchain_specific!(LIGHT, "static/css/themes/light.css");

    // The "dark" theme.
    toolchain_specific!(DARK, "static/css/themes/dark.css");

    // The "ayu" theme.
    toolchain_specific!(AYU, "static/css/themes/ayu.css");
}

/// Files related to the Fira Sans font.
pub(crate) mod fira_sans {
    // The file `FiraSans-Regular.woff2`, the Regular variant of the Fira Sans font in woff2.
    unversioned!(REGULAR, "static/fonts/FiraSans-Regular.woff2");

    // The file `FiraSans-Medium.woff2`, the Medium variant of the Fira Sans font in woff2.
    unversioned!(MEDIUM, "static/fonts/FiraSans-Medium.woff2");

    // The file `FiraSans-LICENSE.txt`, the license text for the Fira Sans font.
    unversioned!(LICENSE, "static/fonts/FiraSans-LICENSE.txt");
}

/// Files related to the Source Serif 4 font.
pub(crate) mod source_serif_4 {
    // The file `SourceSerif4-Regular.ttf.woff2`, the Regular variant of the Source Serif 4 font in
    // woff2.
    unversioned!(REGULAR, "static/fonts/SourceSerif4-Regular.ttf.woff2");

    // The file `SourceSerif4-Bold.ttf.woff2`, the Bold variant of the Source Serif 4 font in
    // woff2.
    unversioned!(BOLD, "static/fonts/SourceSerif4-Bold.ttf.woff2");

    // The file `SourceSerif4-It.ttf.woff2`, the Italic variant of the Source Serif 4 font in
    // woff2.
    unversioned!(ITALIC, "static/fonts/SourceSerif4-It.ttf.woff2");

    // The file `SourceSerif4-LICENSE.txt`, the license text for the Source Serif 4 font.
    unversioned!(LICENSE, "static/fonts/SourceSerif4-LICENSE.md");
}

/// Files related to the Source Code Pro font.
pub(crate) mod source_code_pro {
    // The file `SourceCodePro-Regular.ttf.woff2`, the Regular variant of the Source Code Pro font
    // in woff2.
    unversioned!(REGULAR, "static/fonts/SourceCodePro-Regular.ttf.woff2");

    // The file `SourceCodePro-Semibold.ttf.woff2`, the Semibold variant of the Source Code Pro
    // font in woff2.
    unversioned!(SEMIBOLD, "static/fonts/SourceCodePro-Semibold.ttf.woff2");

    // The file `SourceCodePro-It.ttf.woff2`, the Italic variant of the Source Code Pro font in
    // woff2.
    unversioned!(ITALIC, "static/fonts/SourceCodePro-It.ttf.woff2");

    // The file `SourceCodePro-LICENSE.txt`, the license text of the Source Code Pro font.
    unversioned!(LICENSE, "static/fonts/SourceCodePro-LICENSE.txt");
}

/// Files related to the Nanum Barun Gothic font.
///
/// These files are used to avoid some legacy CJK serif fonts in Windows.
///
/// Note that the Noto Sans KR font, which was used previously but was not very readable on Windows,
/// has been replaced by the Nanum Barun Gothic font. This is due to Windows' implementation of font
/// rendering that distorts OpenType fonts too much.
///
/// The font files were generated with these commands:
///
/// ```sh
/// pyftsubset NanumBarunGothic.ttf \
/// --unicodes=U+AC00-D7AF,U+1100-11FF,U+3130-318F,U+A960-A97F,U+D7B0-D7FF \
/// --output-file=NanumBarunGothic.ttf.woff2 --flavor=woff2
/// ```
pub(crate) mod nanum_barun_gothic {
    // The file `NanumBarunGothic.ttf.woff2`, the Regular variant of the Nanum Barun Gothic font.
    unversioned!(REGULAR, "static/fonts/NanumBarunGothic.ttf.woff2");

    // The file `NanumBarunGothic-LICENSE.txt`, the license text of the Nanum Barun Gothic font.
    unversioned!(LICENSE, "static/fonts/NanumBarunGothic-LICENSE.txt");
}

/// Files related to the sidebar in rustdoc sources.
pub(crate) mod sidebar {
    // File script to handle sidebar.
    toolchain_specific!(SOURCE_SCRIPT, "static/js/source-script.js");
}
