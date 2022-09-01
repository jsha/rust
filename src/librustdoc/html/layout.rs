use std::path::PathBuf;

use rustc_data_structures::fx::FxHashMap;

use crate::error::Error;
use crate::externalfiles::ExternalHtml;
use crate::html::format::{Buffer, Print};
use crate::html::render::{ensure_trailing_slash, StylePath};
use crate::html::static_files;

use askama::Template;

#[derive(Clone)]
pub(crate) struct Layout {
    pub(crate) logo: String,
    pub(crate) favicon: String,
    pub(crate) external_html: ExternalHtml,
    pub(crate) default_settings: FxHashMap<String, String>,
    pub(crate) krate: String,
    /// The given user css file which allow to customize the generated
    /// documentation theme.
    pub(crate) css_file_extension: Option<PathBuf>,
    /// If true, then scrape-examples.js will be included in the output HTML file
    pub(crate) scrape_examples_extension: bool,
}

pub(crate) struct Page<'a> {
    pub(crate) title: &'a str,
    pub(crate) css_class: &'a str,
    pub(crate) root_path: &'a str,
    pub(crate) static_root_path: Option<&'a str>,
    pub(crate) description: &'a str,
    pub(crate) keywords: &'a str,
    pub(crate) resource_suffix: &'a str,
}

impl<'a> Page<'a> {
    pub(crate) fn get_static_root_path(&self) -> &str {
        self.static_root_path.unwrap_or(self.root_path)
    }
}

#[derive(Template)]
#[template(path = "page.html")]
struct PageLayout<'a> {
    static_root_path: &'a str,
    page: &'a Page<'a>,
    layout: &'a Layout,
    font_preloads: Vec<&'static static_files::StaticFile>,

    storage_js: &'static static_files::StaticFile,
    main_js: &'static static_files::StaticFile,
    source_script_js: &'static static_files::StaticFile,
    scrape_examples_js: &'static static_files::StaticFile,

    // crates_js: &'static static_files::StaticFile,
    // source_files_js: &'static static_files::StaticFile,
    // sidebar_items_js: &'static static_files::StaticFile,
    favicon_svg: &'static static_files::StaticFile,
    favicon_png_16: &'static static_files::StaticFile,
    favicon_png_32: &'static static_files::StaticFile,

    rust_logo_svg: &'static static_files::StaticFile,
    wheel_svg: &'static static_files::StaticFile,

    css: Vec<&'static static_files::StaticFile>,
    themes: Vec<String>,
    sidebar: String,
    content: String,
    krate_with_trailing_slash: String,
    pub(crate) rustdoc_version: &'a str,
}

pub(crate) fn render<T: Print, S: Print>(
    layout: &Layout,
    page: &Page<'_>,
    sidebar: S,
    t: T,
    style_files: &[StylePath],
) -> String {
    let static_root_path = page.get_static_root_path();
    let krate_with_trailing_slash = ensure_trailing_slash(&layout.krate).to_string();
    let mut themes: Vec<String> = style_files
        .iter()
        .map(StylePath::basename)
        .collect::<Result<_, Error>>()
        .unwrap_or_default();
    themes.sort();
    let css = vec![&static_files::NORMALIZE_CSS, &static_files::RUSTDOC_CSS];

    let rustdoc_version = rustc_interface::util::version_str().unwrap_or("unknown version");
    let content = Buffer::html().to_display(t); // Note: This must happen before making the sidebar.
    let sidebar = Buffer::html().to_display(sidebar);
    PageLayout {
        static_root_path,
        page,
        layout,
        font_preloads: vec![
            &static_files::source_serif_4::REGULAR,
            &static_files::source_serif_4::BOLD,
            &static_files::fira_sans::REGULAR,
            &static_files::fira_sans::MEDIUM,
            &static_files::source_code_pro::REGULAR,
            &static_files::source_code_pro::SEMIBOLD,
        ],

        storage_js: &static_files::STORAGE_JS,
        main_js: &static_files::MAIN_JS,
        source_script_js: &static_files::sidebar::SOURCE_SCRIPT,
        scrape_examples_js: &static_files::SCRAPE_EXAMPLES_JS,

        favicon_svg: &static_files::RUST_FAVICON_SVG,
        favicon_png_16: &static_files::RUST_FAVICON_PNG_16,
        favicon_png_32: &static_files::RUST_FAVICON_PNG_32,

        rust_logo_svg: &static_files::RUST_LOGO_SVG,
        wheel_svg: &static_files::WHEEL_SVG,

        css,
        themes,
        sidebar,
        content,
        krate_with_trailing_slash,
        rustdoc_version,
    }
    .render()
    .unwrap()
}

pub(crate) fn redirect(url: &str) -> String {
    // <script> triggers a redirect before refresh, so this is fine.
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="refresh" content="0;URL={url}">
    <title>Redirection</title>
</head>
<body>
    <p>Redirecting to <a href="{url}">{url}</a>...</p>
    <script>location.replace("{url}" + location.search + location.hash);</script>
</body>
</html>"##,
        url = url,
    )
}
