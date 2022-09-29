//! Static files bundled with documentation output.
//!
//! All the static files are included here for centralized access in case anything other than the
//! HTML rendering code (say, the theme checker) needs to access one of these files.

use rustc_data_structures::fx::FxHasher;
use std::hash::Hasher;
use std::path::PathBuf;
use std::{fmt, str};

pub(crate) struct StaticFile {
    pub(crate) filename: &'static str,
    pub(crate) bytes: &'static [u8],
}

impl StaticFile {
    pub(crate) fn minified(&self) -> Vec<u8> {
        if self.filename.ends_with(".css") {
            minifier::css::minify(str::from_utf8(self.bytes).unwrap()).unwrap().to_string().into()
        } else if self.filename.ends_with(".js") {
            minifier::js::minify(str::from_utf8(self.bytes).unwrap()).to_string().into()
        } else {
            self.bytes.to_owned()
        }
    }

    pub(crate) fn output_filename(&self) -> PathBuf {
        static_filename(self.filename, self.bytes)
    }
}

/// The Display implementation for a StaticFile outputs its filename. This makes it
/// convenient to interpolate static files into HTML templates.
impl fmt::Display for StaticFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output_filename().display())
    }
}

/// Insert the provided suffix into a filename just before the extension.
pub(crate) fn suffix_path(filename: &str, suffix: &str) -> PathBuf {
    // We use splitn vs Path::extension here because we might get a filename
    // like `style.min.css` and we want to process that into
    // `style-suffix.min.css`.  Path::extension would just return `css`
    // which would result in `style.min-suffix.css` which isn't what we
    // want.
    let (base, ext) = filename.split_once('.').unwrap();
    let filename = format!("{}{}.{}", base, suffix, ext);
    filename.into()
}

pub(crate) fn static_filename(filename: &str, contents: &[u8]) -> PathBuf {
    let filename = filename.rsplit("/").next().unwrap();
    suffix_path(filename, &static_suffix(contents))
}

fn static_suffix(bytes: &[u8]) -> String {
    let mut hasher = FxHasher::default();
    hasher.write(bytes);
    format!("-{:016x}", hasher.finish())
}

macro_rules! include_static {
    ($file:literal) => {
        crate::html::static_files::StaticFile { filename: $file, bytes: include_bytes!($file) }
    };
}

/// A list of all static files for writing into the static.files/ directory.
pub(crate) static STATIC_FILES_LIST: [&'static StaticFile; 38] = [
    &STATIC_FILES.rustdoc_css,
    &STATIC_FILES.settings_css,
    &STATIC_FILES.noscript_css,
    &STATIC_FILES.normalize_css,
    &STATIC_FILES.main_js,
    &STATIC_FILES.search_js,
    &STATIC_FILES.settings_js,
    &STATIC_FILES.storage_js,
    &STATIC_FILES.source_script_js,
    &STATIC_FILES.scrape_examples_js,
    &STATIC_FILES.wheel_svg,
    &STATIC_FILES.clipboard_svg,
    &STATIC_FILES.down_arrow_svg,
    &STATIC_FILES.toggle_minus_png,
    &STATIC_FILES.toggle_plus_png,
    &STATIC_FILES.rust_logo_svg,
    &STATIC_FILES.rust_favicon_svg,
    &STATIC_FILES.rust_favicon_png_16,
    &STATIC_FILES.rust_favicon_png_32,
    &STATIC_FILES.theme_dark_css,
    &STATIC_FILES.theme_light_css,
    &STATIC_FILES.theme_ayu_css,
    &STATIC_FILES.fira_sans_regular,
    &STATIC_FILES.fira_sans_medium,
    &STATIC_FILES.fira_sans_license,
    &STATIC_FILES.source_serif_4_regular,
    &STATIC_FILES.source_serif_4_bold,
    &STATIC_FILES.source_serif_4_italic,
    &STATIC_FILES.source_serif_4_license,
    &STATIC_FILES.source_code_pro_regular,
    &STATIC_FILES.source_code_pro_semibold,
    &STATIC_FILES.source_code_pro_italic,
    &STATIC_FILES.source_code_pro_license,
    &STATIC_FILES.nanum_barun_gothic_regular,
    &STATIC_FILES.nanum_barun_gothic_license,
    &STATIC_FILES.license_mit,
    &STATIC_FILES.license_apache,
    &STATIC_FILES.copyright,
];

/// A struct containing all the static files as fields. This is used for
/// interpolating the calculated filenames into HTML templates.
pub(crate) struct StaticFiles {
    pub rustdoc_css: StaticFile,
    pub settings_css: StaticFile,
    pub noscript_css: StaticFile,
    pub normalize_css: StaticFile,

    pub theme_light_css: StaticFile,
    pub theme_dark_css: StaticFile,
    pub theme_ayu_css: StaticFile,

    pub main_js: StaticFile,
    pub search_js: StaticFile,
    pub settings_js: StaticFile,
    pub source_script_js: StaticFile,
    pub storage_js: StaticFile,
    pub scrape_examples_js: StaticFile,

    pub wheel_svg: StaticFile,
    pub clipboard_svg: StaticFile,
    pub down_arrow_svg: StaticFile,
    pub toggle_minus_png: StaticFile,
    pub toggle_plus_png: StaticFile,
    pub rust_logo_svg: StaticFile,
    pub rust_favicon_svg: StaticFile,
    pub rust_favicon_png_16: StaticFile,
    pub rust_favicon_png_32: StaticFile,

    pub copyright: StaticFile,
    pub license_apache: StaticFile,
    pub license_mit: StaticFile,

    pub fira_sans_regular: StaticFile,
    pub fira_sans_medium: StaticFile,
    pub fira_sans_license: StaticFile,

    pub source_serif_4_regular: StaticFile,
    pub source_serif_4_bold: StaticFile,
    pub source_serif_4_italic: StaticFile,
    pub source_serif_4_license: StaticFile,

    pub source_code_pro_regular: StaticFile,
    pub source_code_pro_semibold: StaticFile,
    pub source_code_pro_italic: StaticFile,
    pub source_code_pro_license: StaticFile,

    pub nanum_barun_gothic_regular: StaticFile,
    pub nanum_barun_gothic_license: StaticFile,
}

pub(crate) static STATIC_FILES: StaticFiles = StaticFiles {
    rustdoc_css: include_static!("static/css/rustdoc.css"),

    // The file contents of `settings.css`, responsible for the items on the settings page.
    settings_css: include_static!("static/css/settings.css"),

    // The file contents of the `noscript.css` file, used in case JS isn't supported or is disabled.
    noscript_css: include_static!("static/css/noscript.css"),

    // The file contents of `normalize.css`, included to even out standard elements between browser
    // implementations.
    normalize_css: include_static!("static/css/normalize.css"),

    // The file contents of `main.js`, which contains the core JavaScript used on documentation pages,
    // including search behavior and docblock folding, among others.
    main_js: include_static!("static/js/main.js"),

    // The file contents of `search.js`, which contains the search behavior.
    search_js: include_static!("static/js/search.js"),

    // The file contents of `settings.js`: which contains the JavaScript used to handle the settings
    // page.
    settings_js: include_static!("static/js/settings.js"),

    source_script_js: include_static!("static/js/source-script.js"),

    // The file contents of `storage.js`: which contains functionality related to browser Local
    // Storage: used to store documentation settings.
    storage_js: include_static!("static/js/storage.js"),

    // The file contents of `scraped-examples.js`: which contains functionality related to the
    // --scrape-examples flag that inserts automatically-found examples of usages of items.
    scrape_examples_js: include_static!("static/js/scrape-examples.js"),

    // The icon used for the settings button.
    wheel_svg: include_static!("static/images/wheel.svg"),

    // The icon used for the "copy path" button.
    clipboard_svg: include_static!("static/images/clipboard.svg"),

    // The icon used for the crate choice combobox.
    down_arrow_svg: include_static!("static/images/down-arrow.svg"),

    // The file contents of `toggle-minus.svg`: include_static!(the icon used for opened toggles.
    toggle_minus_png: include_static!("static/images/toggle-minus.svg"),

    // The icon used for closed toggles.
    toggle_plus_png: include_static!("static/images/toggle-plus.svg"),

    // The license listing for files distributed with documentation output.
    copyright: include_static!("static/COPYRIGHT.txt"),

    // The text of the Apache License, version 2.0.
    license_apache: include_static!("static/LICENSE-APACHE.txt"),

    // The text of the MIT License.
    license_mit: include_static!("static/LICENSE-MIT.txt"),

    // The default icon of the documentation.
    rust_logo_svg: include_static!("static/images/rust-logo.svg"),

    // The default documentation favicons (SVG and PNG fallbacks)
    rust_favicon_svg: include_static!("static/images/favicon.svg"),
    rust_favicon_png_16: include_static!("static/images/favicon-16x16.png"),
    rust_favicon_png_32: include_static!("static/images/favicon-32x32.png"),

    // The "light" theme: selected by default when no setting is available. Used as the basis for
    // the `--check-theme` functionality.
    theme_light_css: include_static!("static/css/themes/light.css"),

    // The "dark" theme.
    theme_dark_css: include_static!("static/css/themes/dark.css"),

    // The "ayu" theme.
    theme_ayu_css: include_static!("static/css/themes/ayu.css"),

    // The Regular variant of the Fira Sans font in woff2.
    fira_sans_regular: include_static!("static/fonts/FiraSans-Regular.woff2"),

    // The Medium variant of the Fira Sans font in woff2.
    fira_sans_medium: include_static!("static/fonts/FiraSans-Medium.woff2"),

    // The license text for the Fira Sans font.
    fira_sans_license: include_static!("static/fonts/FiraSans-LICENSE.txt"),

    // The Regular variant of the Source Serif 4 font in woff2.
    source_serif_4_regular: include_static!("static/fonts/SourceSerif4-Regular.ttf.woff2"),

    // The Bold variant of the Source Serif 4 font in woff2.
    source_serif_4_bold: include_static!("static/fonts/SourceSerif4-Bold.ttf.woff2"),

    // The Italic variant of the Source Serif 4 font in woff2.
    source_serif_4_italic: include_static!("static/fonts/SourceSerif4-It.ttf.woff2"),

    // The license text for the Source Serif 4 font.
    source_serif_4_license: include_static!("static/fonts/SourceSerif4-LICENSE.md"),

    // The Regular variant of the Source Code Pro font in woff2.
    source_code_pro_regular: include_static!("static/fonts/SourceCodePro-Regular.ttf.woff2"),

    // The Semibold variant of the Source Code Pro font in woff2.
    source_code_pro_semibold: include_static!("static/fonts/SourceCodePro-Semibold.ttf.woff2"),

    // The Italic variant of the Source Code Pro font in woff2.
    source_code_pro_italic: include_static!("static/fonts/SourceCodePro-It.ttf.woff2"),

    // The license text of the Source Code Pro font.
    source_code_pro_license: include_static!("static/fonts/SourceCodePro-LICENSE.txt"),

    /// Files related to the Nanum Barun Gothic font.
    ///
    /// These files are used to avoid some legacy CJK serif fonts in Windows.
    ///
    /// Note that the Noto Sans KR font: which was used previously but was not very readable on Windows,
    /// has been replaced by the Nanum Barun Gothic font. This is due to Windows' implementation of font
    /// rendering that distorts OpenType fonts too much.
    ///
    /// The font files were generated with these commands:
    ///
    /// ```sh
    /// pyftsubset NanumBarunGothic.ttf \
    /// --unicodes=U+AC00-D7AF:U+1100-11FF,U+3130-318F,U+A960-A97F,U+D7B0-D7FF \
    /// --output-file=NanumBarunGothic.ttf.woff2 --flavor=woff2
    /// ```
    // The Regular variant of the Nanum Barun Gothic font.
    nanum_barun_gothic_regular: include_static!("static/fonts/NanumBarunGothic.ttf.woff2"),

    // The license text of the Nanum Barun Gothic font.
    nanum_barun_gothic_license: include_static!("static/fonts/NanumBarunGothic-LICENSE.txt"),
};

pub(crate) static SCRAPE_EXAMPLES_HELP_MD: &str = include_str!("static/js/scrape-examples.js");
