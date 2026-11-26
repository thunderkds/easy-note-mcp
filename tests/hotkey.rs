//! HUN-260: `easy-note hotkey` prints the commands that bind a key to `easy-note quick` for the
//! desktop in `$XDG_CURRENT_DESKTOP` (FR-025). It writes nothing and runs no command. Tests use
//! a temp HOME, EASY_NOTE_DIR and a stub-only PATH.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A stub that would leave a marker file if `hotkey` ever ran it.
const TRIPWIRE: &str = "#!/bin/sh\necho ran >> \"$HOME/tripwire\"\n";

type Snapshot = Vec<(PathBuf, Vec<u8>)>;

fn snapshot(root: &Path) -> Snapshot {
    let mut all = vec![];
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                all.push((path.clone(), vec![]));
                stack.push(path);
            } else {
                all.push((path.clone(), std::fs::read(&path).unwrap()));
            }
        }
    }
    all.sort();
    all
}

fn hotkey(desktop: Option<&str>) -> (Output, Snapshot, Snapshot) {
    let dir = tempfile::tempdir().unwrap();
    let (home, bin) = (dir.path().join("home"), dir.path().join("bin"));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    for name in [
        "gsettings",
        "kwriteconfig5",
        "kwriteconfig6",
        "easy-note",
        "notify-send",
    ] {
        let path = bin.join(name);
        std::fs::write(&path, TRIPWIRE).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = snapshot(dir.path());
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy-note"));
    cmd.arg("hotkey")
        .env_clear()
        .env("HOME", &home)
        .env("EASY_NOTE_DIR", dir.path().join("notes"))
        .env("PATH", &bin)
        .current_dir(&home);
    if let Some(desktop) = desktop {
        cmd.env("XDG_CURRENT_DESKTOP", desktop);
    }
    let out = cmd.output().unwrap();
    let after = snapshot(dir.path());
    (out, before, after)
}

fn text(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn ac7_gnome_gets_gsettings_commands_for_a_custom_keybinding() {
    for desktop in ["GNOME", "ubuntu:GNOME", "Unity:gnome"] {
        let (o, _, _) = hotkey(Some(desktop));
        assert_eq!(o.status.code(), Some(0), "{desktop}");
        let t = text(&o);
        assert!(
            t.contains(
                "gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings"
            ),
            "{t}"
        );
        assert!(t.contains("command 'easy-note quick'"), "{t}");
        assert!(t.contains("binding '<Control><Alt>n'"), "{t}");
        assert!(t.contains("name 'easy-note quick'"), "{t}");
        // existing shortcuts must not be wiped without a warning
        assert!(
            t.contains(
                "gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings"
            ),
            "{t}"
        );
    }
}

#[test]
fn ac7_kde_gets_the_system_settings_path_and_no_gsettings() {
    let (o, _, _) = hotkey(Some("KDE"));
    assert_eq!(o.status.code(), Some(0));
    let t = text(&o);
    assert!(t.contains("System Settings"), "{t}");
    assert!(t.contains("easy-note quick"), "{t}");
    assert!(!t.contains("gsettings"), "{t}");
    assert!(!t.contains("kwriteconfig"), "{t}");
}

#[test]
fn ac7_any_other_desktop_or_none_gets_the_generic_text() {
    for desktop in [Some("sway"), Some("X-Cinnamon"), Some(""), None] {
        let (o, _, _) = hotkey(desktop);
        assert_eq!(o.status.code(), Some(0), "{desktop:?}");
        let t = text(&o);
        assert!(t.contains("easy-note quick"), "{t}");
        assert!(!t.contains("gsettings set"), "{t}");
        assert!(!t.contains("System Settings"), "{t}");
    }
}

#[test]
fn ac7_hotkey_writes_nothing_and_runs_no_command() {
    for desktop in [Some("GNOME"), Some("KDE"), None] {
        let (o, before, after) = hotkey(desktop);
        assert_eq!(o.status.code(), Some(0));
        assert_eq!(before, after, "{desktop:?}: the file system is unchanged");
        assert!(
            !after.iter().any(|(p, _)| p.ends_with("tripwire")),
            "{desktop:?}: a command ran"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// HUN-303: `install hotkey` / `uninstall hotkey` and the `setup` step (FR-054).
//
// The real `gsettings` is never run: it writes the desktop settings through D-Bus even with a temp
// HOME. Every run clears the environment and has only this stub on PATH.
// ---------------------------------------------------------------------------------------------

/// A `gsettings` that keeps its values as files in `$STUB_STATE` and logs every call.
/// `fail` there makes each call exit 1; `readonly` makes `set` and `reset` exit 0 and change
/// nothing, as `gsettings` does when dconf cannot commit.
const GSETTINGS: &str = r#"#!/bin/sh
S="$STUB_STATE"
printf '%s\n' "$*" >> "$S/calls"
if [ -f "$S/fail" ]; then echo "stub failure" >&2; exit 1; fi
if [ "$1" = list-recursively ]; then
  if [ -f "$S/recursive" ]; then /bin/cat "$S/recursive"; fi
  exit 0
fi
case "$2" in
  org.gnome.settings-daemon.plugins.media-keys) file="$S/$3"; def="@as []" ;;
  org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:*)
    id=$(printf '%s' "${2#*:}" | /usr/bin/tr / _); file="$S/$id$3"; def="''" ;;
  *) echo "No such schema" >&2; exit 1 ;;
esac
case "$1" in
  get) if [ -f "$file" ]; then /bin/cat "$file"; else printf '%s\n' "$def"; fi ;;
  set) if [ ! -f "$S/readonly" ]; then printf '%s\n' "$4" > "$file"; fi ;;
  reset) if [ ! -f "$S/readonly" ]; then /bin/rm -f "$file"; fi ;;
esac
"#;

const BASE: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/";

fn ours() -> String {
    format!("{BASE}easy-note/")
}

fn custom(n: u8) -> String {
    format!("{BASE}custom{n}/")
}

struct Desk {
    dir: tempfile::TempDir,
}

impl Desk {
    fn new() -> Desk {
        let desk = Desk {
            dir: tempfile::tempdir().unwrap(),
        };
        for d in ["home", "bin", "state", "proc/1", "xdg"] {
            std::fs::create_dir_all(desk.root().join(d)).unwrap();
        }
        std::fs::write(desk.root().join("proc/1/comm"), "bash\n").unwrap();
        let stub = desk.root().join("bin/gsettings");
        std::fs::write(&stub, GSETTINGS).unwrap();
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
        desk
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn state(&self, name: &str) -> PathBuf {
        self.root().join("state").join(name)
    }
    fn put(&self, name: &str, value: &str) {
        std::fs::write(self.state(name), format!("{value}\n")).unwrap();
    }
    fn list(&self, paths: &[String]) {
        let quoted: Vec<String> = paths.iter().map(|p| format!("'{p}'")).collect();
        self.put("custom-keybindings", &format!("[{}]", quoted.join(", ")));
    }
    fn entry(&self, path: &str, name: &str, command: &str, binding: &str) {
        let id = path.replace('/', "_");
        self.put(&format!("{id}name"), &format!("'{name}'"));
        self.put(&format!("{id}command"), &format!("'{command}'"));
        self.put(&format!("{id}binding"), &format!("'{binding}'"));
    }
    /// The stored value, or nothing when the key is not set.
    fn get(&self, path: &str, key: &str) -> Option<String> {
        let id = path.replace('/', "_");
        self.read(&format!("{id}{key}"))
    }
    fn read(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.state(name))
            .ok()
            .map(|t| t.trim_end().to_string())
    }
    fn calls(&self) -> Vec<String> {
        self.read("calls")
            .map(|t| t.lines().map(String::from).collect())
            .unwrap_or_default()
    }
    fn writes(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|c| c.starts_with("set ") || c.starts_with("reset "))
            .collect()
    }
    fn run_on(&self, desktop: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_easy-note"))
            .args(args)
            .env_clear()
            .env("PATH", self.root().join("bin"))
            .env("HOME", self.root().join("home"))
            .env("XDG_CONFIG_HOME", self.root().join("xdg"))
            .env("EASY_NOTE_DIR", self.root().join("notes"))
            .env("EASY_NOTE_PROC_DIR", self.root().join("proc"))
            .env("STUB_STATE", self.root().join("state"))
            .env("XDG_CURRENT_DESKTOP", desktop)
            .current_dir(self.root().join("home"))
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    }
    fn run(&self, args: &[&str]) -> Output {
        self.run_on("ubuntu:GNOME", args)
    }
    fn install(&self) -> Output {
        self.run(&["install", "hotkey"])
    }
    fn uninstall(&self) -> Output {
        self.run(&["uninstall", "hotkey"])
    }
}

fn ok(o: &Output) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}{}",
        text(o),
        String::from_utf8_lossy(&o.stderr)
    );
    text(o)
}

fn failed(o: &Output) -> String {
    assert_eq!(o.status.code(), Some(1), "{}", text(o));
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn install_adds_the_shortcut_and_writes_its_keys_before_the_list() {
    let desk = Desk::new();
    let out = ok(&desk.install());
    assert!(out.contains("Ctrl+Alt+N"), "{out}");
    assert_eq!(
        desk.read("custom-keybindings"),
        Some(format!("['{}']", ours()))
    );
    assert_eq!(desk.get(&ours(), "name").unwrap(), "'easy-note quick'");
    assert_eq!(desk.get(&ours(), "command").unwrap(), "'easy-note quick'");
    assert_eq!(desk.get(&ours(), "binding").unwrap(), "'<Control><Alt>n'");
    let writes = desk.writes();
    assert_eq!(writes.len(), 4, "{writes:?}");
    assert!(writes[3].contains(" custom-keybindings "), "{writes:?}");
}

#[test]
fn install_keeps_every_other_shortcut_in_order() {
    let desk = Desk::new();
    desk.list(&[custom(1), custom(0)]);
    desk.entry(&custom(0), "Term", "kitty", "<Super>t");
    desk.entry(&custom(1), "Off", "foo", "");
    desk.put(
        "recursive",
        "org.gnome.desktop.wm.keybindings maximize ['<Super>Up']\n\
         org.gnome.shell.keybindings x ['<Control><Alt><Shift>n', '<Control>n']",
    );
    ok(&desk.install());
    assert_eq!(
        desk.read("custom-keybindings"),
        Some(format!("['{}', '{}', '{}']", custom(1), custom(0), ours()))
    );
    assert_eq!(desk.get(&custom(0), "binding").unwrap(), "'<Super>t'");
    assert_eq!(desk.get(&custom(1), "command").unwrap(), "'foo'");
}

#[test]
fn a_second_install_writes_nothing() {
    let desk = Desk::new();
    ok(&desk.install());
    let first = desk.writes();
    let out = ok(&desk.install());
    assert_eq!(desk.writes(), first, "{out}");
    assert!(out.contains("<Control><Alt>n"), "{out}");
}

#[test]
fn a_shortcut_of_the_person_that_runs_quick_is_left_alone() {
    for (path, command) in [
        (custom(0), "/usr/bin/easy-note quick"),
        (ours(), "easy-note quick"),
    ] {
        let desk = Desk::new();
        desk.list(std::slice::from_ref(&path));
        desk.entry(&path, "Mine", command, "<Super>n");
        let out = ok(&desk.install());
        assert!(out.contains("<Super>n"), "{out}");
        assert!(desk.writes().is_empty(), "{:?}", desk.writes());
    }
}

#[test]
fn our_shortcut_with_a_cleared_key_stays_off() {
    let desk = Desk::new();
    desk.list(&[ours()]);
    desk.entry(&ours(), "easy-note quick", "easy-note quick", "");
    let out = ok(&desk.install());
    assert!(out.contains("no key"), "{out}");
    assert!(desk.writes().is_empty(), "{:?}", desk.writes());
}

#[test]
fn a_half_done_entry_is_completed_once() {
    let listed = Desk::new();
    listed.list(&[ours()]);
    let unlisted = Desk::new();
    unlisted.entry(&ours(), "easy-note quick", "easy-note quick", "");
    for desk in [listed, unlisted] {
        ok(&desk.install());
        assert_eq!(
            desk.read("custom-keybindings"),
            Some(format!("['{}']", ours()))
        );
        assert_eq!(desk.get(&ours(), "binding").unwrap(), "'<Control><Alt>n'");
    }
}

#[test]
fn a_used_key_in_a_custom_shortcut_is_not_taken() {
    for binding in ["<Control><Alt>n", "<Alt><Primary>N", "<Ctrl><Mod1>n"] {
        let desk = Desk::new();
        desk.list(&[custom(0)]);
        desk.entry(&custom(0), "Nautilus", "nautilus", binding);
        let out = ok(&desk.install());
        assert!(out.contains("Nautilus"), "{binding}: {out}");
        assert!(desk.writes().is_empty(), "{binding}: {:?}", desk.writes());
    }
}

#[test]
fn a_used_key_in_a_gnome_setting_is_not_taken() {
    let desk = Desk::new();
    desk.put(
        "recursive",
        "org.gnome.desktop.wm.keybindings maximize ['<Super>Up', '<Alt><Control>n']",
    );
    let out = ok(&desk.install());
    assert!(
        out.contains("org.gnome.desktop.wm.keybindings maximize"),
        "{out}"
    );
    assert!(desk.writes().is_empty(), "{:?}", desk.writes());
}

#[test]
fn another_command_at_our_path_is_kept() {
    let desk = Desk::new();
    desk.list(&[ours()]);
    desk.entry(&ours(), "Mine", "gedit", "<Super>g");
    for out in [ok(&desk.install()), ok(&desk.uninstall())] {
        assert!(out.contains("kept"), "{out}");
    }
    assert!(desk.writes().is_empty(), "{:?}", desk.writes());
}

#[test]
fn a_list_that_cannot_be_read_is_never_written() {
    for bad in ["nonsense", "['/a/'", "['/a/' '/b/']", "[1, 2]", ""] {
        let desk = Desk::new();
        desk.put("custom-keybindings", bad);
        failed(&desk.install());
        failed(&desk.uninstall());
        assert!(desk.writes().is_empty(), "{bad}: {:?}", desk.writes());
    }
}

#[test]
fn a_failing_or_missing_gsettings_is_an_error() {
    let desk = Desk::new();
    desk.put("fail", "");
    let err = failed(&desk.install());
    assert!(err.contains("stub failure"), "{err}");

    let desk = Desk::new();
    std::fs::remove_file(desk.root().join("bin/gsettings")).unwrap();
    let err = failed(&desk.install());
    assert!(err.contains("gsettings"), "{err}");
}

#[test]
fn a_write_that_does_not_stick_is_an_error() {
    let desk = Desk::new();
    desk.put("readonly", "");
    let err = failed(&desk.install());
    assert!(err.contains("not saved"), "{err}");

    let desk = Desk::new();
    ok(&desk.install());
    desk.put("readonly", "");
    let err = failed(&desk.uninstall());
    assert!(err.contains("not saved"), "{err}");
}

#[test]
fn uninstall_removes_only_our_shortcut() {
    let desk = Desk::new();
    desk.list(&[custom(0)]);
    desk.entry(&custom(0), "Quick", "easy-note quick", "<Super>n");
    let out = ok(&desk.uninstall());
    assert!(out.contains("not installed"), "{out}");
    assert!(desk.writes().is_empty(), "{:?}", desk.writes());

    desk.list(&[custom(0), ours(), custom(1)]);
    desk.entry(&ours(), "easy-note quick", "easy-note quick", "<Super>e");
    ok(&desk.uninstall());
    assert_eq!(
        desk.read("custom-keybindings"),
        Some(format!("['{}', '{}']", custom(0), custom(1)))
    );
    for key in ["name", "command", "binding"] {
        assert_eq!(desk.get(&ours(), key), None, "{key}");
    }
    assert_eq!(desk.get(&custom(0), "binding").unwrap(), "'<Super>n'");

    let alone = Desk::new();
    ok(&alone.install());
    ok(&alone.uninstall());
    assert_eq!(alone.read("custom-keybindings").unwrap(), "@as []");
}

#[test]
fn another_desktop_runs_no_command() {
    for desktop in ["KDE", "sway", ""] {
        let desk = Desk::new();
        for action in ["install", "uninstall"] {
            let out = ok(&desk.run_on(desktop, &[action, "hotkey"]));
            assert!(out.contains("easy-note hotkey"), "{desktop}: {out}");
        }
        assert!(desk.calls().is_empty(), "{desktop}: {:?}", desk.calls());
    }
}

#[test]
fn the_gnome_text_names_install_hotkey() {
    let desk = Desk::new();
    let out = ok(&desk.run(&["hotkey"]));
    assert!(out.contains("easy-note install hotkey"), "{out}");
    assert!(desk.calls().is_empty(), "{:?}", desk.calls());
}

#[test]
fn setup_binds_the_key() {
    let desk = Desk::new();
    let out = ok(&desk.run(&["setup"]));
    assert!(out.contains("hotkey:"), "{out}");
    assert_eq!(
        desk.get(&ours(), "binding").as_deref(),
        Some("'<Control><Alt>n'")
    );
}

#[test]
fn a_failed_shortcut_step_fails_setup_but_not_the_other_steps() {
    let desk = Desk::new();
    desk.put("fail", "");
    let o = desk.run(&["setup"]);
    let err = failed(&o);
    assert!(err.contains("setup: hotkey failed"), "{err}");
    let out = text(&o);
    assert!(out.contains("Using Claude Code?"), "{out}");
    assert!(desk
        .root()
        .join("home/.local/share/applications/easy-note.desktop")
        .exists());
}
