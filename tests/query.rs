//! HUN-272 query engine (port of Python HUN-256 tests/test_query.py): list, search and stats over a
//! notes dir (FR-010/011/015/016/017, NFR-007, D10).
//!
//! Most tests call the `*_in` functions on a `Store` over the fixture or over a temp dir. Tests that
//! read the env (the entry points that open the store from `EASY_NOTE_DIR`, and the reader-zone
//! test) take `ENV_LOCK` and run on a temp `HOME`. No test opens the real `~/easy-notes`.
//!
//! Rust port notes. A bad `limit` of another type, a `datetime` passed as a date and `query=None`
//! cannot compile in Rust; the nearest checks are `limit = 0`, a date-time string as a date and an
//! empty query. Dates are passed as strings, so `20261114` (a Python int) becomes the string
//! "20261114", which is refused like any text that is not `YYYY-MM-DD`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::DateTime;
use easy_note_mcp::core::query::{
    self, Filters, Page, QueryError, DEFAULT_LIMIT, MAX_LIMIT, SNIPPET_MAX,
};
use easy_note_mcp::core::store::Store;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/notes_basic")
}

/// A temp dir that is removed when the test ends.
struct Sandbox(PathBuf);

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let dir =
            std::env::temp_dir().join(format!("easy-note-query-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Sandbox(dir)
    }
    fn notes(&self) -> PathBuf {
        self.0.join("notes")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn store(root: &Path) -> Store {
    Store::system(root.to_path_buf())
}

fn basic() -> Store {
    store(&fixture())
}

/// One note file written by hand, so a test controls `created` exactly.
#[derive(Default)]
struct W<'a> {
    body: &'a str,
    tags: &'a [&'a str],
    repo: Option<&'a str>,
    task: Option<&'a str>,
    source: Option<&'a str>,
}

fn write_note(root: &Path, id: &str, created: &str, w: W) {
    let mut lines = vec![
        "---".to_string(),
        "easy-note: 1".to_string(),
        format!("id: \"{id}\""),
        format!("created: \"{created}\""),
    ];
    if let Some(repo) = w.repo {
        lines.push(format!("repo: \"{repo}\""));
    }
    if let Some(task) = w.task {
        lines.push(format!("task: \"{task}\""));
    }
    lines.push(format!("tags: {}", serde_json::to_string(w.tags).unwrap()));
    lines.push(format!("source: \"{}\"", w.source.unwrap_or("cli")));
    lines.push("---".to_string());
    let body = if w.body.is_empty() {
        "hello\n"
    } else {
        w.body
    };
    lines.push(body.to_string());
    let folder = root.join(&id[..4]).join(&id[4..6]);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join(format!("{id}.md")), lines.join("\n")).unwrap();
}

fn plain(root: &Path, id: &str, created: &str) {
    write_note(root, id, created, W::default());
}

// An oracle that reads the fixture files with plain string work, never through the code under test.
struct Raw {
    id: String,
    created: String,
    repo: Option<String>,
    task: Option<String>,
    source: String,
    tags: Vec<String>,
    body: String,
    local_day: String,
    instant: i64,
}

fn raw_notes() -> Vec<Raw> {
    let mut paths = vec![];
    for year in std::fs::read_dir(fixture()).unwrap().flatten() {
        for month in std::fs::read_dir(year.path()).unwrap().flatten() {
            for file in std::fs::read_dir(month.path()).unwrap().flatten() {
                paths.push(file.path());
            }
        }
    }
    paths.sort();
    let mut out = vec![];
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap();
        let Some(rest) = text.strip_prefix("---\n") else {
            continue;
        };
        let (front, body) = rest.split_once("\n---\n").unwrap();
        let mut meta: BTreeMap<&str, String> = BTreeMap::new();
        for line in front.lines() {
            let (k, v) = line.split_once(": ").unwrap();
            meta.insert(k, v.trim_matches('"').to_string());
        }
        let created = meta["created"].clone();
        out.push(Raw {
            id: meta["id"].clone(),
            repo: meta.get("repo").cloned(),
            task: meta.get("task").cloned(),
            source: meta["source"].clone(),
            tags: serde_json::from_str(&meta["tags"]).unwrap(),
            body: body.to_string(),
            local_day: created[..10].to_string(),
            instant: DateTime::parse_from_rfc3339(&created)
                .unwrap()
                .timestamp(),
            created,
        });
    }
    out
}

fn ids(page: &Page) -> Vec<String> {
    page.items.iter().map(|i| i.id.clone()).collect()
}

fn expected(pred: impl Fn(&Raw) -> bool) -> Vec<String> {
    let mut hits: Vec<Raw> = raw_notes().into_iter().filter(|n| pred(n)).collect();
    hits.sort_by(|a, b| b.instant.cmp(&a.instant).then(a.id.cmp(&b.id)));
    hits.into_iter().map(|n| n.id).collect()
}

/// The fixture cycles 5 topics by the last 4 hex digits of the ID.
fn topic(id: &str) -> u32 {
    u32::from_str_radix(&id[id.len() - 4..], 16).unwrap() % 5
}

fn f() -> Filters {
    Filters::default()
}

fn list(s: &Store, filters: &Filters, limit: usize) -> Page {
    query::list_notes_in(s, filters, limit).unwrap()
}

fn search(s: &Store, q: &str, filters: &Filters, limit: usize) -> Page {
    query::search_in(s, q, filters, limit).unwrap()
}

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

fn counts(pairs: Vec<(String, usize)>) -> Vec<(String, usize)> {
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for (k, _) in pairs {
        *map.entry(k).or_default() += 1;
    }
    let mut out: Vec<(String, usize)> = map.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

// AC1 — AND of words, any case, in title, body or tags
#[test]
fn search_needs_every_word_in_any_case() {
    let st = basic();
    let both = search(&st, "retry firefox", &f(), 50);
    assert_eq!(ids(&both), expected(|n| topic(&n.id) == 0));
    assert_eq!(both.total, 6);
    assert_eq!(ids(&search(&st, "RETRY FiReFoX", &f(), 50)), ids(&both));
    assert_eq!(search(&st, "retry nonexistentword", &f(), 20).total, 0);
}

#[test]
fn search_matches_title_body_and_tags() {
    let st = basic();
    assert_eq!(search(&st, "navigation", &f(), 20).total, 6); // body only
    assert_eq!(search(&st, "changelog", &f(), 20).total, 6); // body only (topic 3)
    let mut got = ids(&search(&st, "idea/ui", &f(), 50)); // tag only
    let mut want: Vec<String> = raw_notes()
        .into_iter()
        .filter(|n| n.tags.iter().any(|t| t == "idea/ui"))
        .map(|n| n.id)
        .collect();
    got.sort();
    want.sort();
    assert_eq!(got, want);
    // "flaky" is a tag of topic 0 and the title word of topic 1
    let mut topics: Vec<u32> = ids(&search(&st, "flaky", &f(), 50))
        .iter()
        .map(|i| topic(i))
        .collect();
    topics.sort();
    topics.dedup();
    assert_eq!(topics, vec![0, 1]);
}

#[test]
fn search_words_may_sit_in_different_places() {
    let st = basic();
    // "firefox" is in the title and a tag, "navigation" only in the body
    assert_eq!(search(&st, "firefox navigation", &f(), 20).total, 6);
    assert_eq!(search(&st, "ci navigation", &f(), 20).total, 0);
}

#[test]
fn search_folds_case_beyond_ascii() {
    let st = basic();
    assert_eq!(search(&st, "STRASSE", &f(), 20).total, 6); // "ß" folds to "ss"
    assert_eq!(search(&st, "straße", &f(), 20).total, 6);
    assert_eq!(search(&st, "CAFÉ résumé", &f(), 20).total, 6);
}

#[test]
fn empty_query_is_an_error_not_every_note() {
    let st = basic();
    for bad in ["", "   ", "\n\t"] {
        let err = query::search_in(&st, bad, &f(), 20).unwrap_err();
        assert!(matches!(err, QueryError::EmptyQuery), "{bad:?}");
        assert_eq!(err.to_string(), "query is empty");
    }
}

// AC2 — each filter narrows the result
#[test]
fn each_filter_matches_the_oracle() {
    let st = basic();
    type Case = (Filters, Box<dyn Fn(&Raw) -> bool>);
    let cases: Vec<(&str, Case)> = vec![
        (
            "repo",
            (
                Filters {
                    repo: s("beta"),
                    ..f()
                },
                Box::new(|n| n.repo.as_deref() == Some("beta")),
            ),
        ),
        (
            "task",
            (
                Filters {
                    task: s("T101"),
                    ..f()
                },
                Box::new(|n| n.task.as_deref() == Some("T101")),
            ),
        ),
        (
            "tag",
            (
                Filters {
                    tag: s("firefox"),
                    ..f()
                },
                Box::new(|n| n.tags.iter().any(|t| t == "firefox")),
            ),
        ),
        (
            "source",
            (
                Filters {
                    source: s("hook"),
                    ..f()
                },
                Box::new(|n| n.source == "hook"),
            ),
        ),
    ];
    for (name, (filters, pred)) in cases {
        let want = expected(pred);
        assert!(!want.is_empty() && want.len() < 30, "{name}");
        assert_eq!(ids(&list(&st, &filters, 50)), want, "{name}");
        // search applies the same filters (FR-011): every fixture note has "#" in its title
        assert_eq!(ids(&search(&st, "#", &filters, 50)), want, "{name}");
    }
    let gamma = Filters {
        repo: s("gamma"),
        ..f()
    };
    assert_eq!(
        ids(&search(&st, "release", &gamma, 50)),
        expected(|n| topic(&n.id) == 3 && n.repo.as_deref() == Some("gamma"))
    );
}

#[test]
fn tag_filter_is_exact_for_hierarchical_tags() {
    let st = basic();
    let tag = |t: &str| Filters {
        tag: s(t),
        ..f()
    };
    assert_eq!(list(&st, &tag("idea/ui"), 20).total, 6);
    assert_eq!(
        ids(&list(&st, &tag("idea"), 50)),
        expected(|n| n.tags.iter().any(|t| t == "idea"))
    );
    assert_eq!(list(&st, &tag("ui"), 20).total, 0);
    assert_eq!(list(&st, &tag("idea/"), 20).total, 0);
    assert_eq!(
        ids(&list(&st, &tag("FIREFOX"), 50)),
        expected(|n| n.tags.iter().any(|t| t == "firefox"))
    );
}

#[test]
fn filters_combine_with_and() {
    let st = basic();
    let both = |repo: &str, source: &str| Filters {
        repo: s(repo),
        source: s(source),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &both("alpha", "cli"), 50)),
        expected(|n| n.repo.as_deref() == Some("alpha") && n.source == "cli")
    );
    assert_eq!(
        ids(&list(&st, &both("alpha", "hook"), 50)),
        expected(|n| n.repo.as_deref() == Some("alpha") && n.source == "hook")
    );
}

#[test]
fn unknown_filter_value_gives_nothing() {
    let page = list(
        &basic(),
        &Filters {
            repo: s("no-such-repo"),
            ..f()
        },
        20,
    );
    assert_eq!((page.items.len(), page.total, page.truncated), (0, 0, false));
}

#[test]
fn date_filters_use_local_dates() {
    let st = basic();
    let day = |d: &str| Filters {
        day: s(d),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &day("2026-11-05"), 50)),
        expected(|n| n.local_day == "2026-11-05")
    );
    let since = Filters {
        since: s("2026-11-10"),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &since, 50)),
        expected(|n| n.local_day.as_str() >= "2026-11-10")
    );
    let until = Filters {
        until: s("2026-10-31"),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &until, 50)),
        expected(|n| n.local_day.as_str() <= "2026-10-31")
    );
    let window = Filters {
        since: s("2026-11-03"),
        until: s("2026-11-06"),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &window, 50)),
        expected(|n| ("2026-11-03"..="2026-11-06").contains(&n.local_day.as_str()))
    );
    let clash = Filters {
        day: s("2026-11-04"),
        since: s("2026-11-05"),
        ..f()
    };
    assert!(list(&st, &clash, 20).items.is_empty()); // filters AND
    let as_date = chrono::NaiveDate::from_ymd_opt(2026, 11, 5).unwrap().to_string();
    assert_eq!(
        list(&st, &day(&as_date), 20).total,
        list(&st, &day("2026-11-05"), 20).total
    );
}

#[test]
fn bad_date_is_an_error() {
    let st = basic();
    let bads = [
        "2026-13-01",
        "yesterday",
        "",
        "14/11/2026",
        "20261114",
        "2026-1-5",
        "2026-11-05T00:00:00", // a date-time is not a date
    ];
    for bad in bads {
        for name in ["since", "until", "day"] {
            let filters = match name {
                "since" => Filters {
                    since: s(bad),
                    ..f()
                },
                "until" => Filters {
                    until: s(bad),
                    ..f()
                },
                _ => Filters { day: s(bad), ..f() },
            };
            let err = query::list_notes_in(&st, &filters, 20).unwrap_err();
            assert!(matches!(err, QueryError::BadDate { .. }), "{name} {bad:?}");
            assert_eq!(err.to_string(), format!("{name} must be a date like 2026-11-14"));
        }
    }
}

// AC3 — a note counts for the local day written in its offset, not its UTC day
#[test]
fn late_evening_note_counts_for_its_local_day() {
    let sb = Sandbox::new("late");
    let root = sb.notes();
    let late = |id, created, body| {
        write_note(
            &root,
            id,
            created,
            W {
                body,
                ..W::default()
            },
        )
    };
    late("20261114-233000-aaaa", "2026-11-14T23:30:00+07:00", "late\n");
    late("20261114-210000-bbbb", "2026-11-14T21:00:00-05:00", "west\n"); // UTC: 15th
    late("20261115-013000-cccc", "2026-11-15T01:30:00+07:00", "early\n"); // UTC: 14th
    let st = store(&root);
    let day = |d: &str| Filters {
        day: s(d),
        ..f()
    };
    assert_eq!(
        ids(&list(&st, &day("2026-11-14"), 20)),
        ["20261114-210000-bbbb", "20261114-233000-aaaa"]
    );
    assert_eq!(
        ids(&list(&st, &day("2026-11-15"), 20)),
        ["20261115-013000-cccc"]
    );
    let since = Filters {
        since: s("2026-11-15"),
        ..f()
    };
    assert_eq!(list(&st, &since, 20).total, 1);
    let until = Filters {
        until: s("2026-11-14"),
        ..f()
    };
    assert_eq!(list(&st, &until, 20).total, 2);
}

#[test]
fn fixture_notes_around_midnight() {
    let st = basic();
    let day = |d: &str| ids(&list(&st, &Filters { day: s(d), ..f() }, 50));
    assert!(day("2026-11-03").contains(&"20261103-011500-0008".to_string())); // 01:15+07:00, UTC 2nd
    assert!(!day("2026-11-02").contains(&"20261103-011500-0008".to_string()));
    assert!(day("2026-10-29").contains(&"20261029-230000-0003".to_string())); // 23:00-05:00, UTC 30th
    assert!(!day("2026-10-30").contains(&"20261029-230000-0003".to_string()));
}

#[test]
fn result_does_not_depend_on_the_reader_zone() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let st = basic();
    let want = expected(|n| n.local_day == "2026-11-03");
    assert!(!want.is_empty());
    for zone in [
        "UTC",
        "Asia/Ho_Chi_Minh",
        "Pacific/Auckland",
        "America/Los_Angeles",
    ] {
        std::env::set_var("TZ", zone);
        let got = ids(&list(
            &st,
            &Filters {
                day: s("2026-11-03"),
                ..f()
            },
            50,
        ));
        assert_eq!(got, want, "{zone}");
    }
    std::env::remove_var("TZ");
}

// AC4 — newest instant first, ties by ID, same output twice
#[test]
fn whole_fixture_is_newest_first_by_utc_instant() {
    let st = basic();
    let got = list(&st, &f(), 50);
    assert_eq!(ids(&got), expected(|_| true));
    assert_eq!(got.total, 30);
    // the order is by instant, not by the wall-clock text of `created`
    let instants: Vec<i64> = got
        .items
        .iter()
        .map(|i| DateTime::parse_from_rfc3339(&i.created).unwrap().timestamp())
        .collect();
    let mut sorted = instants.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(instants, sorted);
}

#[test]
fn equal_instants_are_ordered_by_id() {
    let sb = Sandbox::new("ties");
    let root = sb.notes();
    // same instant in three offsets, plus a newer and an older note
    plain(&root, "20261114-100000-cccc", "2026-11-14T10:00:00+07:00");
    plain(&root, "20261114-030000-bbbb", "2026-11-14T03:00:00+00:00");
    plain(&root, "20261113-220000-aaaa", "2026-11-13T22:00:00-05:00");
    plain(&root, "20261114-120000-dddd", "2026-11-14T12:00:00+07:00");
    plain(&root, "20261114-080000-eeee", "2026-11-14T08:00:00+07:00");
    assert_eq!(
        ids(&list(&store(&root), &f(), 20)),
        [
            "20261114-120000-dddd",
            "20261113-220000-aaaa",
            "20261114-030000-bbbb",
            "20261114-100000-cccc",
            "20261114-080000-eeee",
        ]
    );
}

#[test]
fn clash_suffix_ids_tie_break_as_text() {
    let sb = Sandbox::new("clash");
    let root = sb.notes();
    for suffix in ["", "-2", "-10"] {
        plain(
            &root,
            &format!("20261114-100000-abcd{suffix}"),
            "2026-11-14T10:00:00+07:00",
        );
    }
    assert_eq!(
        ids(&list(&store(&root), &f(), 20)),
        [
            "20261114-100000-abcd",
            "20261114-100000-abcd-10",
            "20261114-100000-abcd-2"
        ]
    );
}

#[test]
fn same_output_twice() {
    let st = basic();
    let repo_since = Filters {
        repo: s("alpha"),
        since: s("2026-11-01"),
        ..f()
    };
    let ci = Filters {
        tag: s("ci"),
        ..f()
    };
    let calls: Vec<Box<dyn Fn() -> String>> = vec![
        Box::new(|| json(&list(&st, &f(), 50))),
        Box::new(|| json(&list(&st, &repo_since, 20))),
        Box::new(|| json(&search(&st, "retry firefox", &f(), 50))),
        Box::new(|| json(&search(&st, "note", &ci, 20))),
        Box::new(|| json(&query::stats_in(&st, &f()).unwrap())),
    ];
    for call in calls {
        assert_eq!(call(), call());
    }
}

#[test]
fn listing_order_ignores_the_order_the_files_are_found() {
    let sb = Sandbox::new("order");
    let (first, second) = (sb.0.join("a"), sb.0.join("b"));
    for (root, order) in [(&first, [1, 2, 3]), (&second, [3, 1, 2])] {
        for n in order {
            plain(root, &format!("20261114-10000{n}-abcd"), "2026-11-14T10:00:00+07:00");
        }
    }
    assert_eq!(
        json(&list(&store(&first), &f(), 20)),
        json(&list(&store(&second), &f(), 20))
    );
}

// AC5 — limit cap 50, total, truncated
#[test]
fn limit_is_capped_at_fifty() {
    let sb = Sandbox::new("cap");
    let root = sb.notes();
    for n in 0..60 {
        plain(
            &root,
            &format!("20261114-10{n:02}00-abcd"),
            &format!("2026-11-14T10:{n:02}:00+07:00"),
        );
    }
    let st = store(&root);
    let big = list(&st, &f(), 1000);
    assert_eq!(big.items.len(), 50);
    assert_eq!(MAX_LIMIT, 50);
    assert_eq!(big.total, 60);
    assert!(big.truncated);
    assert_eq!(search(&st, "hello", &f(), 51).items.len(), 50);
    let small = list(&st, &f(), 5);
    assert_eq!((small.items.len(), small.total, small.truncated), (5, 60, true));
    assert_eq!(ids(&small), ids(&big)[..5]);
}

#[test]
fn default_limit_and_truncated_flag() {
    let st = basic();
    assert_eq!(DEFAULT_LIMIT, 20);
    let default = list(&st, &f(), DEFAULT_LIMIT);
    assert_eq!(default.items.len(), 20);
    assert_eq!((default.total, default.truncated), (30, true));
    let exact = list(&st, &f(), 30);
    assert_eq!((exact.items.len(), exact.total, exact.truncated), (30, 30, false));
    assert!(!list(&st, &f(), 50).truncated);
}

#[test]
fn total_counts_every_match_not_the_page() {
    let page = search(&basic(), "retry firefox", &f(), 2);
    assert_eq!((page.items.len(), page.total, page.truncated), (2, 6, true));
}

#[test]
fn zero_limit_is_an_error() {
    let st = basic();
    let err = query::list_notes_in(&st, &f(), 0).unwrap_err();
    assert!(matches!(err, QueryError::BadLimit));
    assert_eq!(err.to_string(), "limit must be a whole number of 1 or more");
    assert!(matches!(
        query::search_in(&st, "retry", &f(), 0).unwrap_err(),
        QueryError::BadLimit
    ));
}

// AC6 — item shape and snippet
#[test]
fn item_has_the_frozen_fields() {
    let item = search(&basic(), "retry firefox", &f(), 1).items.remove(0);
    let value = serde_json::to_value(&item).unwrap();
    let mut keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        ["created", "id", "repo", "snippet", "tags", "task", "title"]
    );
    let raw = raw_notes().into_iter().find(|n| n.id == item.id).unwrap();
    assert_eq!(item.created, raw.created);
    assert_eq!(item.repo, raw.repo);
    assert_eq!(item.tags, raw.tags);
    assert!(item.title.starts_with("Retry bug on Firefox #"));
}

#[test]
fn item_without_repo_or_task_has_none() {
    let sb = Sandbox::new("norepo");
    plain(&sb.notes(), "20261114-100000-abcd", "2026-11-14T10:00:00+07:00");
    let item = list(&store(&sb.notes()), &f(), 20).items.remove(0);
    assert!(item.repo.is_none() && item.task.is_none() && item.tags.is_empty());
    let value = serde_json::to_value(&item).unwrap();
    assert!(value["repo"].is_null() && value["task"].is_null());
}

fn one_note(name: &str, body: &str, tags: &[&str]) -> (Sandbox, Store) {
    let sb = Sandbox::new(name);
    write_note(
        &sb.notes(),
        "20261114-100000-abcd",
        "2026-11-14T10:00:00+07:00",
        W {
            body,
            tags,
            ..W::default()
        },
    );
    let st = store(&sb.notes());
    (sb, st)
}

#[test]
fn snippet_is_cut_around_the_first_hit() {
    let body = format!("Title line\n\n{}NEEDLE here {}\n", "alpha ".repeat(200), "omega ".repeat(200));
    let (_sb, st) = one_note("snip", &body, &[]);
    let snippet = search(&st, "needle", &f(), 20).items.remove(0).snippet;
    assert!(snippet.chars().count() <= SNIPPET_MAX);
    assert_eq!(SNIPPET_MAX, 200);
    assert!(snippet.contains("NEEDLE here"));
    assert!(snippet.starts_with('…') && snippet.ends_with('…'));
    assert!(!snippet.contains('\n'));
}

#[test]
fn snippet_picks_the_earliest_of_several_words() {
    let body = format!("Title\n\n{}second {}first \n", "x ".repeat(300), "y ".repeat(50));
    let (_sb, st) = one_note("snip2", &body, &[]);
    let snippet = search(&st, "first second", &f(), 20).items.remove(0).snippet;
    assert!(snippet.contains("second"));
}

#[test]
fn snippet_never_exceeds_the_limit_at_any_hit_position() {
    let sb = Sandbox::new("snippos");
    for n in [0usize, 30, 39, 40, 41, 150, 159, 160, 161, 199, 200, 201, 500] {
        let body = format!("{}needle{}\n", "q".repeat(n), "q".repeat(400usize.saturating_sub(n)));
        write_note(
            &sb.notes(),
            &format!("20261114-1000{:02}-{:04x}", n / 100, n % 100),
            "2026-11-14T10:00:00+07:00",
            W {
                body: &body,
                ..W::default()
            },
        );
    }
    let page = search(&store(&sb.notes()), "needle", &f(), 50);
    assert_eq!(page.total, 13);
    for item in page.items {
        assert!(item.snippet.chars().count() <= 200, "{}", item.id);
        assert!(item.snippet.contains("needle"), "{}", item.id);
    }
}

#[test]
fn short_body_snippet_is_the_whole_body() {
    let (_sb, st) = one_note("short", "Title\n\nsmall body here\n", &[]);
    assert_eq!(
        search(&st, "small", &f(), 20).items[0].snippet,
        "Title small body here"
    );
}

#[test]
fn list_snippet_is_the_start_of_the_body() {
    let (_sb, st) = one_note("liststart", &format!("{}\n", "z".repeat(500)), &[]);
    assert_eq!(
        list(&st, &f(), 20).items[0].snippet,
        format!("{}…", "z".repeat(199))
    );
}

#[test]
fn hit_only_in_tags_gives_the_start_of_the_body() {
    let (_sb, st) = one_note("tagonly", "Body text\n", &["onlytag"]);
    assert_eq!(search(&st, "onlytag", &f(), 20).items[0].snippet, "Body text");
}

#[test]
fn snippet_finds_hit_after_characters_that_change_length_when_folded() {
    let (_sb, st) = one_note("fold", &format!("{} target here\n", "ß".repeat(300)), &[]);
    let snippet = search(&st, "TARGET", &f(), 20).items.remove(0).snippet;
    assert!(snippet.contains("target") && snippet.chars().count() <= 200);
}

#[test]
fn title_is_cut_to_120() {
    let (_sb, st) = one_note("title", &format!("{}\n", "t".repeat(300)), &[]);
    assert_eq!(list(&st, &f(), 20).items[0].title.chars().count(), 120);
}

// AC7 — stats
#[test]
fn stats_counts_match_the_oracle() {
    let notes = raw_notes();
    let got = query::stats_in(&basic(), &f()).unwrap();
    assert_eq!(got.total, 30);
    let pairs = |c: &query::Counts| c.as_pairs().to_vec();
    assert_eq!(
        pairs(&got.by_repo),
        counts(notes.iter().filter_map(|n| n.repo.clone().map(|r| (r, 1))).collect())
    );
    assert_eq!(
        pairs(&got.by_task),
        counts(notes.iter().filter_map(|n| n.task.clone().map(|t| (t, 1))).collect())
    );
    assert_eq!(
        pairs(&got.by_tag),
        counts(notes.iter().flat_map(|n| n.tags.iter().map(|t| (t.clone(), 1))).collect())
    );
    assert_eq!(
        pairs(&got.by_source),
        counts(notes.iter().map(|n| (n.source.clone(), 1)).collect())
    );
    assert_eq!(
        pairs(&got.by_month),
        counts(notes.iter().map(|n| (n.local_day[..7].to_string(), 1)).collect())
    );
    assert_eq!(
        pairs(&got.by_month),
        [("2026-11".to_string(), 23), ("2026-10".to_string(), 7)]
    );
    let days: Vec<&str> = notes.iter().map(|n| n.local_day.as_str()).collect();
    assert_eq!(got.oldest.as_deref(), days.iter().min().copied());
    assert_eq!(got.oldest.as_deref(), Some("2026-10-28"));
    assert_eq!(got.newest.as_deref(), days.iter().max().copied());
    assert_eq!(got.newest.as_deref(), Some("2026-11-13"));
    assert_eq!(got.by_repo.as_pairs().iter().map(|p| p.1).sum::<usize>(), 30);
    // a third of the notes have no task
    assert_eq!(got.by_task.as_pairs().iter().map(|p| p.1).sum::<usize>(), 20);
}

#[test]
fn stats_counts_are_ordered_by_count_then_name() {
    let got = query::stats_in(&basic(), &f()).unwrap();
    for c in [&got.by_repo, &got.by_task, &got.by_tag, &got.by_source, &got.by_month] {
        let pairs = c.as_pairs().to_vec();
        let mut sorted = pairs.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        assert_eq!(pairs, sorted);
    }
}

#[test]
fn stats_counts_serialize_as_a_map_in_count_order() {
    let got = query::stats_in(&basic(), &f()).unwrap();
    assert!(json(&got).contains(r#""by_month":{"2026-11":23,"2026-10":7}"#));
}

#[test]
fn stats_lists_skipped_files_with_path_and_reason() {
    let got = query::stats_in(&basic(), &f()).unwrap();
    assert_eq!(got.skipped.len(), 1);
    assert_eq!(
        got.skipped[0].path,
        fixture().join("2026/11/20261110-100000-bad0.md").to_string_lossy()
    );
    assert_eq!(got.skipped[0].reason, "no front matter");
}

#[test]
fn stats_returns_no_note_text() {
    let dump = json(&query::stats_in(&basic(), &f()).unwrap());
    for note in raw_notes() {
        for sentence in note.body.lines().filter(|l| !l.trim().is_empty()) {
            assert!(!dump.contains(sentence), "{sentence}");
        }
    }
    assert!(!dump.contains("SKIPPED-FILE-TEXT")); // the unreadable file's text stays out too
    assert!(!dump.contains("Firefox")); // a title word
}

#[test]
fn stats_follow_the_filters() {
    let filters = Filters {
        repo: s("alpha"),
        ..f()
    };
    let got = query::stats_in(&basic(), &filters).unwrap();
    let alpha: Vec<Raw> = raw_notes()
        .into_iter()
        .filter(|n| n.repo.as_deref() == Some("alpha"))
        .collect();
    assert_eq!(got.total, alpha.len());
    assert_eq!(alpha.len(), 10);
    assert_eq!(got.by_repo.as_pairs(), [("alpha".to_string(), 10)]);
    assert_eq!(
        got.by_source.as_pairs().to_vec(),
        counts(alpha.iter().map(|n| (n.source.clone(), 1)).collect())
    );
    assert_eq!(got.skipped.len(), 1); // skipped files are reported whatever the filter
}

#[test]
fn stats_of_an_empty_dir() {
    let sb = Sandbox::new("emptystats");
    std::fs::create_dir_all(sb.notes()).unwrap();
    assert_eq!(
        json(&query::stats_in(&store(&sb.notes()), &f()).unwrap()),
        r#"{"total":0,"by_repo":{},"by_task":{},"by_tag":{},"by_source":{},"by_month":{},"oldest":null,"newest":null,"skipped":[]}"#
    );
}

#[test]
fn stats_counts_a_repeated_tag_once_per_note() {
    let (_sb, st) = one_note("duptag", "x\n", &["a", "a", "b"]);
    let got = query::stats_in(&st, &f()).unwrap();
    assert_eq!(
        got.by_tag.as_pairs(),
        [("a".to_string(), 1), ("b".to_string(), 1)]
    );
}

#[test]
fn list_and_search_leave_skipped_files_out() {
    let st = basic();
    let page = list(&st, &f(), 50);
    assert!(!ids(&page).contains(&"20261110-100000-bad0".to_string()));
    assert_eq!(page.total, 30);
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let mtime = entry.metadata().unwrap().modified().unwrap();
                out.insert(path.clone(), (std::fs::read(&path).unwrap(), mtime));
            }
        }
    }
    out
}

#[test]
fn queries_change_nothing_on_disk() {
    let sb = Sandbox::new("readonly");
    let copy = sb.0.join("copy");
    copy_tree(&fixture(), &copy);
    let before = snapshot(&copy);
    let st = store(&copy);
    list(&st, &f(), 50);
    search(&st, "retry", &f(), 20);
    query::stats_in(&st, &f()).unwrap();
    assert_eq!(before, snapshot(&copy));
}

// The entry points open the store from the env, like the CLI and MCP do.
fn with_env_dir<T>(dir: &Path, home: &Path, run: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("HOME", home);
    std::env::set_var("EASY_NOTE_DIR", dir);
    let out = run();
    std::env::remove_var("EASY_NOTE_DIR");
    out
}

#[test]
fn entry_points_read_the_notes_dir_from_the_env() {
    let sb = Sandbox::new("env");
    let (fix, home) = (fixture(), sb.0.join("home"));
    std::fs::create_dir_all(&home).unwrap();
    with_env_dir(&fix, &home, || {
        let page = query::search("retry firefox", &f(), 50).unwrap();
        assert_eq!(page.total, 6);
        assert_eq!(query::list_notes(&f(), DEFAULT_LIMIT).unwrap().total, 30);
        assert_eq!(query::stats(&f()).unwrap().total, 30);
    });
}

#[test]
fn missing_notes_dir_is_empty_not_an_error() {
    let sb = Sandbox::new("missing");
    let home = sb.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    with_env_dir(&sb.0.join("never-created"), &home, || {
        let page = query::list_notes(&f(), DEFAULT_LIMIT).unwrap();
        assert_eq!((page.items.len(), page.total, page.truncated), (0, 0, false));
        assert_eq!(query::stats(&f()).unwrap().total, 0);
    });
}
