//! `dashboard`: in a build with the `window` feature and a display, open easy-note's own window
//! (FR-042, DDR 0009). Otherwise open `<notes dir>/easy-note.base` in Obsidian (FR-034, DDR 0006):
//! Obsidian draws the window. `install obsidian` writes the file (FR-035). The programs
//! `xdg-mime` and `xdg-open` run as argv lists, never through a shell (NFR-001).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

use crate::core::config;

/// The file name in the notes dir.
pub const BASE_NAME: &str = "easy-note.base";

/// The dashboard: an Obsidian Bases file. The views mirror the design's groupings (day, project,
/// task, tag, source). The filter keeps only notes: a file in a `YYYY/MM` folder, or any file with
/// the `easy-note` property (so it also works when the notes dir sits inside a larger vault).
pub const BASE_FILE: &str = r#"filters:
  and:
    - '/(^|\/)\d{4}\/\d{2}$/.matches(file.folder)'
    - 'file.hasProperty("easy-note")'
views:
  - type: table
    name: Today
    filters:
      and:
        - 'file.name.startsWith(today().format("YYYYMMDD"))'
    order:
      - note.created
      - file.name
      - note.repo
      - note.task
      - note.tags
      - note.source
    sort:
      - property: file.name
        direction: DESC
  - type: table
    name: All notes
    order:
      - note.created
      - file.name
      - note.repo
      - note.task
      - note.tags
      - note.source
    sort:
      - property: file.name
        direction: DESC
  - type: table
    name: By project
    groupBy:
      property: note.repo
      direction: ASC
    order:
      - note.created
      - file.name
      - note.task
      - note.tags
    sort:
      - property: file.name
        direction: DESC
  - type: table
    name: By task
    groupBy:
      property: note.task
      direction: ASC
    order:
      - note.created
      - file.name
      - note.repo
      - note.tags
    sort:
      - property: file.name
        direction: DESC
  - type: table
    name: By tag
    groupBy:
      property: note.tags
      direction: ASC
    order:
      - note.created
      - file.name
      - note.repo
      - note.task
    sort:
      - property: file.name
        direction: DESC
  - type: table
    name: By source
    groupBy:
      property: note.source
      direction: ASC
    order:
      - note.created
      - file.name
      - note.repo
      - note.task
    sort:
      - property: file.name
        direction: DESC
"#;

/// Everything but the unreserved characters is encoded.
const QUERY_VALUE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// `obsidian://open?path=<percent-encoded absolute path>`.
pub fn obsidian_uri(path: &Path) -> String {
    let path = path.to_string_lossy();
    format!(
        "obsidian://open?path={}",
        utf8_percent_encode(&path, QUERY_VALUE)
    )
}

/// What `run` does, decided from the file and the `obsidian` scheme handler.
#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    NoDashboard,
    OpenInObsidian,
    OpenFolder,
}

/// `handler` is what `xdg-mime query default x-scheme-handler/obsidian` printed ("" when none).
pub fn choose(handler: &str, base_exists: bool) -> Plan {
    if !base_exists {
        Plan::NoDashboard
    } else if handler.trim().is_empty() {
        Plan::OpenFolder
    } else {
        Plan::OpenInObsidian
    }
}

/// What `xdg-mime` names as the `obsidian://` handler ("" when none or no `xdg-mime`).
pub(crate) fn handler() -> String {
    Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/obsidian"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn open(target: &str) -> Result<(), String> {
    match Command::new("xdg-open").arg(target).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("xdg-open failed ({status})")),
        Err(e) => Err(format!("cannot run xdg-open: {e}")),
    }
}

/// The own window opens only in a `window` build with `WAYLAND_DISPLAY` or `DISPLAY` set (non-empty).
pub fn use_window(built: bool, wayland: Option<&str>, display: Option<&str>) -> bool {
    let set = |v: Option<&str>| v.is_some_and(|v| !v.is_empty());
    built && (set(wayland) || set(display))
}

/// `use_window` for this binary and this process's environment.
pub fn window_here() -> bool {
    let wayland = std::env::var("WAYLAND_DISPLAY").ok();
    let display = std::env::var("DISPLAY").ok();
    use_window(
        cfg!(feature = "window"),
        wayland.as_deref(),
        display.as_deref(),
    )
}

/// `note` is the id the window selects (`dashboard --note`, already checked); the Obsidian path
/// opens the dashboard without it.
pub fn run(note: Option<&str>) -> Result<(), String> {
    if window_here() {
        match super::window::open(note) {
            Err(e) if e == super::window::NO_DISPLAY => {}
            done => return done,
        }
    }
    let dir = config::notes_dir().map_err(|e| e.to_string())?;
    let base: PathBuf = std::path::absolute(dir.join(BASE_NAME)).map_err(|e| e.to_string())?;
    match choose(&handler(), base.is_file()) {
        Plan::NoDashboard => Err("no dashboard yet: run easy-note install obsidian".to_string()),
        Plan::OpenInObsidian => {
            if super::install::in_vault_list(&dir) == Some(false) {
                eprintln!(
                    "easy-note: the notes dir is not an Obsidian vault yet: run easy-note setup"
                );
            }
            open(&obsidian_uri(&base))
        }
        Plan::OpenFolder => {
            eprintln!("easy-note: Obsidian is not installed; opening the notes folder instead");
            open(&dir.to_string_lossy())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saphyr::{LoadableYamlNode, ScalarOwned, YamlOwned};

    fn s(v: &str) -> YamlOwned {
        YamlOwned::Value(ScalarOwned::String(v.to_string()))
    }

    fn get<'a>(map: &'a YamlOwned, key: &str) -> Option<&'a YamlOwned> {
        match map {
            YamlOwned::Mapping(m) => m.get(&s(key)),
            _ => None,
        }
    }

    fn strings(list: &YamlOwned) -> Vec<String> {
        match list {
            YamlOwned::Sequence(items) => items
                .iter()
                .filter_map(|i| match i {
                    YamlOwned::Value(ScalarOwned::String(x)) => Some(x.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    }

    #[test]
    fn the_base_file_has_the_designed_views_and_only_real_keys() {
        let docs = YamlOwned::load_from_str(BASE_FILE).expect("valid YAML");
        let root = &docs[0];
        let filters = get(root, "filters").expect("a global filter");
        let rules = strings(get(filters, "and").expect("an `and` filter"));
        assert!(
            rules
                .iter()
                .any(|r| r.contains(r"\d{4}\/\d{2}$/.matches(file.folder)")),
            "{rules:?}"
        );
        assert!(rules.iter().any(|r| r.contains("easy-note")), "{rules:?}");

        let views = match get(root, "views") {
            Some(YamlOwned::Sequence(v)) => v,
            other => panic!("views: {other:?}"),
        };
        let names: Vec<String> = views
            .iter()
            .map(|v| {
                strings(&YamlOwned::Sequence(vec![get(v, "name").unwrap().clone()]))[0].clone()
            })
            .collect();
        for want in ["Today", "All notes", "By project", "By task", "By tag"] {
            assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
        }

        let keys: Vec<String> =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/NOTE_FORMAT.md"))
                .unwrap()
                .lines()
                .skip_while(|l| !l.starts_with("## Front matter keys"))
                .skip(1)
                .take_while(|l| !l.starts_with("##"))
                .filter_map(|l| l.strip_prefix("| `")?.split('`').next().map(str::to_string))
                .collect();
        assert!(keys.contains(&"repo".to_string()), "{keys:?}");
        let mut named = vec![];
        for v in views {
            named.extend(strings(get(v, "order").expect("an order")));
            let sort = get(v, "sort").expect("a sort");
            if let YamlOwned::Sequence(items) = sort {
                named.extend(items.iter().map(|i| {
                    strings(&YamlOwned::Sequence(vec![get(i, "property")
                        .unwrap()
                        .clone()]))[0]
                        .clone()
                }));
            }
            if let Some(g) = get(v, "groupBy") {
                named.push(
                    strings(&YamlOwned::Sequence(vec![get(g, "property")
                        .unwrap()
                        .clone()]))[0]
                        .clone(),
                );
            }
        }
        for prop in named {
            if let Some(key) = prop.strip_prefix("note.") {
                assert!(
                    keys.iter().any(|k| k == key),
                    "{prop} is not a front matter key"
                );
            } else {
                assert!(prop.starts_with("file."), "{prop}");
            }
        }
    }

    #[test]
    fn the_uri_encodes_spaces_and_non_ascii() {
        assert_eq!(
            obsidian_uri(Path::new("/home/a b/ghi é/easy-note.base")),
            "obsidian://open?path=%2Fhome%2Fa%20b%2Fghi%20%C3%A9%2Feasy-note.base"
        );
    }

    #[test]
    fn the_plan_follows_the_file_and_the_handler() {
        assert_eq!(choose("obsidian.desktop\n", false), Plan::NoDashboard);
        assert_eq!(choose("", false), Plan::NoDashboard);
        assert_eq!(choose("obsidian.desktop\n", true), Plan::OpenInObsidian);
        assert_eq!(choose("", true), Plan::OpenFolder);
        assert_eq!(choose(" \n", true), Plan::OpenFolder);
    }
}
