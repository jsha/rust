use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::{Component, Path};
use std::rc::Rc;

use itertools::Itertools;
use rustc_data_structures::flock;
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use serde::Serialize;

use super::{collect_paths_for_type, ensure_trailing_slash, Context, BASIC_KEYWORDS};
use crate::clean::Crate;
use crate::config::{EmitType, RenderOptions};
use crate::docfs::PathError;
use crate::error::Error;
use crate::html::static_files::StaticFile;
use crate::html::{layout, static_files};
use crate::{try_err, try_none};

impl Context<'_> {
    fn write_shared(
        &self,
        resource: &StaticFile,
        contents: impl 'static + Send + AsRef<[u8]>,
        emit: &[EmitType],
    ) -> Result<(), Error> {
        if resource.should_emit(emit) {
            self.shared
                .fs
                .write(resource.output_filename(&self.dst, &self.shared.resource_suffix), contents)
        } else {
            Ok(())
        }
    }

    fn write_minify(
        &self,
        resource: &StaticFile,
        minify: bool,
        emit: &[EmitType],
    ) -> Result<(), Error> {
        if minify {
            let contents = resource.as_str();
            let contents = if resource.extension() == Some(OsStr::new("css")) {
                minifier::css::minify(contents)
                    .map_err(|e| {
                        Error::new(format!("failed to minify CSS file: {}", e), resource.filename)
                    })?
                    .to_string()
            } else {
                minifier::js::minify(contents).to_string()
            };
            self.write_shared(resource, contents, emit)
        } else {
            self.write_shared(resource, resource.as_str(), emit)
        }
    }
}

pub(super) fn write_shared(
    cx: &mut Context<'_>,
    krate: &Crate,
    search_index: String,
    options: &RenderOptions,
) -> Result<(), Error> {
    // Write out the shared files. Note that these are shared among all rustdoc
    // docs placed in the output directory, so this needs to be a synchronized
    // operation with respect to all other rustdocs running around.
    let lock_file = cx.dst.join(".lock");
    let _lock = try_err!(flock::Lock::new(&lock_file, true, true, true), &lock_file);

    // Minified resources are usually toolchain resources. If they're not, they should use `cx.write_minify` directly.
    fn write_minify(
        resource: &StaticFile,
        cx: &Context<'_>,
        options: &RenderOptions,
    ) -> Result<(), Error> {
        cx.write_minify(resource, options.enable_minification, &options.emit)
    }

    // Crate resources should always be dynamic.
    let write_crate = |p: &str, make_content: &dyn Fn() -> Result<Vec<u8>, Error>| {
        let content = make_content()?;
        if options.emit.contains(&crate::config::EmitType::InvocationSpecific) {
            let output_filename = static_files::suffix_path(p, &cx.dst, &cx.shared.resource_suffix);
            cx.shared.fs.write(output_filename, content)
        } else {
            Ok(())
        }
    };

    // Given "foo.svg", return e.g. "url(\"foo1.58.0.svg\")"
    fn ver_url(cx: &Context<'_>, basename: &'static str) -> String {
        format!(
            "url(\"{}\")",
            static_files::suffix_path(basename, &cx.dst, &cx.shared.resource_suffix)
                .to_str()
                .unwrap()
        )
    }

    if options.emit.contains(&EmitType::Toolchain) {
        // We use the AUTOREPLACE mechanism to inject into our static CSS certain
        // values that are only known at doc build time. Since this mechanism is somewhat
        // surprising when reading the code, please limit it to rustdoc.css.
        let mut rustdoc_css_rewritten = static_files::RUSTDOC_CSS
            .as_str()
            .replace(
                "/* AUTOREPLACE: */url(\"toggle-minus.svg\")",
                &ver_url(cx, "toggle-minus.svg"),
            )
            .replace("/* AUTOREPLACE: */url(\"toggle-plus.svg\")", &ver_url(cx, "toggle-plus.svg"))
            .replace("/* AUTOREPLACE: */url(\"down-arrow.svg\")", &ver_url(cx, "down-arrow.svg"));
        if options.enable_minification {
            rustdoc_css_rewritten = minifier::js::minify(&rustdoc_css_rewritten).to_string()
        };
        cx.shared.fs.write(
            static_files::RUSTDOC_CSS.output_filename(&cx.dst, &cx.shared.resource_suffix),
            rustdoc_css_rewritten,
        )?;
    }

    // To avoid "light.css" to be overwritten, we'll first run over the received themes and only
    // then we'll run over the "official" styles.
    let mut themes: FxHashSet<String> = FxHashSet::default();

    for entry in &cx.shared.style_files {
        let theme = entry.basename()?;
        let extension =
            try_none!(try_none!(entry.path.extension(), &entry.path).to_str(), &entry.path);

        // Handle added third-party themes
        let filename = format!("{}.{}", theme, extension);
        write_crate(&filename, &|| Ok(try_err!(fs::read(&entry.path), &entry.path)))?;
        themes.insert(theme.to_owned());
    }

    let mut themes: Vec<&String> = themes.iter().collect();
    themes.sort();

    if cx.include_sources {
        write_minify(&static_files::sidebar::SOURCE_SCRIPT, cx, options)?;
    }

    if let Some(ref css) = cx.shared.layout.css_file_extension {
        let buffer = try_err!(fs::read_to_string(css), css);
        let path =
            static_files::suffix_path(css.to_str().unwrap(), &cx.dst, &cx.shared.resource_suffix);
        cx.shared.fs.write(path, buffer)?;
    }

    for f in &static_files::FILES_UNVERSIONED {
        cx.write_shared(f, f.bytes, &options.emit)?;
    }

    for f in &static_files::FILES_TOOLCHAIN {
        cx.write_shared(f, f.bytes, &options.emit)?;
    }

    fn collect(path: &Path, krate: &str, key: &str) -> io::Result<(Vec<String>, Vec<String>)> {
        let mut ret = Vec::new();
        let mut krates = Vec::new();

        if path.exists() {
            let prefix = format!(r#"{}["{}"]"#, key, krate);
            for line in BufReader::new(File::open(path)?).lines() {
                let line = line?;
                if !line.starts_with(key) {
                    continue;
                }
                if line.starts_with(&prefix) {
                    continue;
                }
                ret.push(line.to_string());
                krates.push(
                    line[key.len() + 2..]
                        .split('"')
                        .next()
                        .map(|s| s.to_owned())
                        .unwrap_or_else(String::new),
                );
            }
        }
        Ok((ret, krates))
    }

    fn collect_json(path: &Path, krate: &str) -> io::Result<(Vec<String>, Vec<String>)> {
        let mut ret = Vec::new();
        let mut krates = Vec::new();

        if path.exists() {
            let prefix = format!("\"{}\"", krate);
            for line in BufReader::new(File::open(path)?).lines() {
                let line = line?;
                if !line.starts_with('"') {
                    continue;
                }
                if line.starts_with(&prefix) {
                    continue;
                }
                if line.ends_with(",\\") {
                    ret.push(line[..line.len() - 2].to_string());
                } else {
                    // Ends with "\\" (it's the case for the last added crate line)
                    ret.push(line[..line.len() - 1].to_string());
                }
                krates.push(
                    line.split('"')
                        .find(|s| !s.is_empty())
                        .map(|s| s.to_owned())
                        .unwrap_or_else(String::new),
                );
            }
        }
        Ok((ret, krates))
    }

    use std::ffi::OsString;

    #[derive(Debug)]
    struct Hierarchy {
        elem: OsString,
        children: FxHashMap<OsString, Hierarchy>,
        elems: FxHashSet<OsString>,
    }

    impl Hierarchy {
        fn new(elem: OsString) -> Hierarchy {
            Hierarchy { elem, children: FxHashMap::default(), elems: FxHashSet::default() }
        }

        fn to_json_string(&self) -> String {
            let mut subs: Vec<&Hierarchy> = self.children.values().collect();
            subs.sort_unstable_by(|a, b| a.elem.cmp(&b.elem));
            let mut files = self
                .elems
                .iter()
                .map(|s| format!("\"{}\"", s.to_str().expect("invalid osstring conversion")))
                .collect::<Vec<_>>();
            files.sort_unstable();
            let subs = subs.iter().map(|s| s.to_json_string()).collect::<Vec<_>>().join(",");
            let dirs = if subs.is_empty() && files.is_empty() {
                String::new()
            } else {
                format!(",[{}]", subs)
            };
            let files = files.join(",");
            let files = if files.is_empty() { String::new() } else { format!(",[{}]", files) };
            format!(
                "[\"{name}\"{dirs}{files}]",
                name = self.elem.to_str().expect("invalid osstring conversion"),
                dirs = dirs,
                files = files
            )
        }
    }

    if cx.include_sources {
        let mut hierarchy = Hierarchy::new(OsString::new());
        for source in cx
            .shared
            .local_sources
            .iter()
            .filter_map(|p| p.0.strip_prefix(&cx.shared.src_root).ok())
        {
            let mut h = &mut hierarchy;
            let mut elems = source
                .components()
                .filter_map(|s| match s {
                    Component::Normal(s) => Some(s.to_owned()),
                    _ => None,
                })
                .peekable();
            loop {
                let cur_elem = elems.next().expect("empty file path");
                if elems.peek().is_none() {
                    h.elems.insert(cur_elem);
                    break;
                } else {
                    let e = cur_elem.clone();
                    h = h.children.entry(cur_elem.clone()).or_insert_with(|| Hierarchy::new(e));
                }
            }
        }

        let dst = cx.dst.join(&format!("source-files{}.js", cx.shared.resource_suffix));
        let make_sources = || {
            let (mut all_sources, _krates) =
                try_err!(collect_json(&dst, krate.name(cx.tcx()).as_str()), &dst);
            all_sources.push(format!(
                r#""{}":{}"#,
                &krate.name(cx.tcx()),
                hierarchy
                    .to_json_string()
                    // All these `replace` calls are because we have to go through JS string for JSON content.
                    .replace('\\', r"\\")
                    .replace('\'', r"\'")
                    // We need to escape double quotes for the JSON.
                    .replace("\\\"", "\\\\\"")
            ));
            all_sources.sort();
            let mut v = String::from("var sourcesIndex = JSON.parse('{\\\n");
            v.push_str(&all_sources.join(",\\\n"));
            v.push_str("\\\n}');\ncreateSourceSidebar();\n");
            Ok(v.into_bytes())
        };
        write_crate("source-files.js", &make_sources)?;
    }

    // Update the search index and crate list.
    let dst = cx.dst.join(&format!("search-index{}.js", cx.shared.resource_suffix));
    let (mut all_indexes, mut krates) =
        try_err!(collect_json(&dst, krate.name(cx.tcx()).as_str()), &dst);
    all_indexes.push(search_index);
    krates.push(krate.name(cx.tcx()).to_string());
    krates.sort();

    // Sort the indexes by crate so the file will be generated identically even
    // with rustdoc running in parallel.
    all_indexes.sort();
    write_crate("search-index.js", &|| {
        let mut v = String::from("var searchIndex = JSON.parse('{\\\n");
        v.push_str(&all_indexes.join(",\\\n"));
        v.push_str(
            r#"\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
"#,
        );
        Ok(v.into_bytes())
    })?;

    write_crate("crates.js", &|| {
        let krates = krates.iter().map(|k| format!("\"{}\"", k)).join(",");
        Ok(format!("window.ALL_CRATES = [{}];", krates).into_bytes())
    })?;

    if options.enable_index_page {
        if let Some(index_page) = options.index_page.clone() {
            let mut md_opts = options.clone();
            md_opts.output = cx.dst.clone();
            md_opts.external_html = (*cx.shared).layout.external_html.clone();

            crate::markdown::render(&index_page, md_opts, cx.shared.edition())
                .map_err(|e| Error::new(e, &index_page))?;
        } else {
            let shared = Rc::clone(&cx.shared);
            let dst = cx.dst.join("index.html");
            let page = layout::Page {
                title: "Index of crates",
                css_class: "mod",
                root_path: "./",
                static_root_path: shared.static_root_path.as_deref(),
                description: "List of crates",
                keywords: BASIC_KEYWORDS,
                resource_suffix: &shared.resource_suffix,
            };

            let content = format!(
                "<h1 class=\"fqn\">\
                     <span class=\"in-band\">List of all crates</span>\
                </h1><ul class=\"crate mod\">{}</ul>",
                krates
                    .iter()
                    .map(|s| {
                        format!(
                            "<li><a class=\"crate mod\" href=\"{}index.html\">{}</a></li>",
                            ensure_trailing_slash(s),
                            s
                        )
                    })
                    .collect::<String>()
            );
            let v = layout::render(&shared.layout, &page, "", content, &shared.style_files);
            shared.fs.write(dst, v)?;
        }
    }

    // Update the list of all implementors for traits
    let dst = cx.dst.join("implementors");
    let cache = cx.cache();
    for (&did, imps) in &cache.implementors {
        // Private modules can leak through to this phase of rustdoc, which
        // could contain implementations for otherwise private types. In some
        // rare cases we could find an implementation for an item which wasn't
        // indexed, so we just skip this step in that case.
        //
        // FIXME: this is a vague explanation for why this can't be a `get`, in
        //        theory it should be...
        let (remote_path, remote_item_type) = match cache.exact_paths.get(&did) {
            Some(p) => match cache.paths.get(&did).or_else(|| cache.external_paths.get(&did)) {
                Some((_, t)) => (p, t),
                None => continue,
            },
            None => match cache.external_paths.get(&did) {
                Some((p, t)) => (p, t),
                None => continue,
            },
        };

        #[derive(Serialize)]
        struct Implementor {
            text: String,
            synthetic: bool,
            types: Vec<String>,
        }

        let implementors = imps
            .iter()
            .filter_map(|imp| {
                // If the trait and implementation are in the same crate, then
                // there's no need to emit information about it (there's inlining
                // going on). If they're in different crates then the crate defining
                // the trait will be interested in our implementation.
                //
                // If the implementation is from another crate then that crate
                // should add it.
                if imp.impl_item.item_id.krate() == did.krate || !imp.impl_item.item_id.is_local() {
                    None
                } else {
                    Some(Implementor {
                        text: imp.inner_impl().print(false, cx).to_string(),
                        synthetic: imp.inner_impl().kind.is_auto(),
                        types: collect_paths_for_type(imp.inner_impl().for_.clone(), cache),
                    })
                }
            })
            .collect::<Vec<_>>();

        // Only create a js file if we have impls to add to it. If the trait is
        // documented locally though we always create the file to avoid dead
        // links.
        if implementors.is_empty() && !cache.paths.contains_key(&did) {
            continue;
        }

        let implementors = format!(
            r#"implementors["{}"] = {};"#,
            krate.name(cx.tcx()),
            serde_json::to_string(&implementors).unwrap()
        );

        let mut mydst = dst.clone();
        for part in &remote_path[..remote_path.len() - 1] {
            mydst.push(part.to_string());
        }
        cx.shared.ensure_dir(&mydst)?;
        mydst.push(&format!("{}.{}.js", remote_item_type, remote_path[remote_path.len() - 1]));

        let (mut all_implementors, _) =
            try_err!(collect(&mydst, krate.name(cx.tcx()).as_str(), "implementors"), &mydst);
        all_implementors.push(implementors);
        // Sort the implementors by crate so the file will be generated
        // identically even with rustdoc running in parallel.
        all_implementors.sort();

        let mut v = String::from("(function() {var implementors = {};\n");
        for implementor in &all_implementors {
            writeln!(v, "{}", *implementor).unwrap();
        }
        v.push_str(
            "if (window.register_implementors) {\
                 window.register_implementors(implementors);\
             } else {\
                 window.pending_implementors = implementors;\
             }",
        );
        v.push_str("})()");
        cx.shared.fs.write(mydst, v)?;
    }
    Ok(())
}
