//! HUN-287: `scripts/build_deb.sh` makes an installable .deb from a prebuilt binary.
//!
//! The test passes `/bin/true` as the binary, so it exercises the real staging, the real
//! `dpkg-shlibdeps` run and the real `dpkg-deb --build` in under a second, without a release
//! build. Everything the app serves (icons, window page, fonts) is embedded in the binary, so
//! the package carries the binary and the docs, and nothing else.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// stdout of a command that must succeed, trimmed.
fn run<S: AsRef<OsStr>>(cmd: &str, args: &[S]) -> String {
    let out = Command::new(cmd).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "{cmd} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn build_deb_packages_the_binary_and_the_docs() {
    // dpkg-deb, dpkg-shlibdeps and dpkg come together in dpkg-dev; the script checks all three.
    if Command::new("dpkg-deb").arg("--version").output().is_err() {
        eprintln!("skipped: dpkg-deb is not on PATH");
        return;
    }

    let out = tempfile::tempdir().unwrap();
    let run_script = Command::new("bash")
        .arg(repo_root().join("scripts/build_deb.sh"))
        .arg("--out")
        .arg(out.path())
        .env("EASY_NOTE_BIN", "/bin/true")
        .current_dir(repo_root())
        .output()
        .unwrap();
    assert!(
        run_script.status.success(),
        "build_deb.sh failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_script.stdout),
        String::from_utf8_lossy(&run_script.stderr)
    );

    let version = env!("CARGO_PKG_VERSION");
    let arch = run("dpkg", &["--print-architecture"]);
    let deb = out.path().join(format!("easy-note_{version}_{arch}.deb"));
    assert!(deb.exists(), "no {} in {:?}", deb.display(), out.path());
    let deb = deb.into_os_string();

    // The control fields a package manager reads, in one call.
    let fields = run(
        "dpkg-deb",
        &[
            OsStr::new("--field"),
            &deb,
            OsStr::new("Package"),
            OsStr::new("Version"),
            OsStr::new("Architecture"),
            OsStr::new("Depends"),
        ],
    );
    for want in [
        "Package: easy-note".to_string(),
        format!("Version: {version}"),
        format!("Architecture: {arch}"),
    ] {
        assert!(
            fields.lines().any(|l| l == want),
            "no {want:?} in:\n{fields}"
        );
    }
    let depends = fields
        .lines()
        .find(|l| l.starts_with("Depends: "))
        .unwrap_or_else(|| panic!("no Depends in:\n{fields}"));
    assert!(
        depends.contains("libc6"),
        "Depends is computed from the binary, so it must name libc6: {depends:?}"
    );

    // The payload: the binary, executable, plus the docs Debian policy asks for.
    let contents = run("dpkg-deb", &[OsStr::new("--contents"), &deb]);
    let bin_line = contents
        .lines()
        .find(|l| l.ends_with("./usr/bin/easy-note"))
        .unwrap_or_else(|| panic!("no ./usr/bin/easy-note in:\n{contents}"));
    assert!(
        bin_line.starts_with("-rwxr-xr-x"),
        "the binary must be executable: {bin_line:?}"
    );
    for want in [
        "./usr/share/doc/easy-note/copyright",
        "./usr/share/doc/easy-note/README.md",
        "./usr/share/doc/easy-note/changelog.gz",
    ] {
        assert!(
            contents.lines().any(|l| l.ends_with(want)),
            "no {want} in:\n{contents}"
        );
    }
    // And nothing else: a file the package carries is a file the person did not ask for.
    let files = contents
        .lines()
        .filter(|l| l.starts_with('-'))
        .collect::<Vec<_>>();
    assert_eq!(
        files.len(),
        4,
        "the package holds the binary and three doc files, nothing else:\n{contents}"
    );

    // No maintainer script: per-user setup belongs to `easy-note setup`, which runs as the
    // person, and a postinst runs as root (HUN-277 cut list). `--info` lists the control
    // archive's own members, so it shows a maintainer script if one is there.
    let info = run("dpkg-deb", &[OsStr::new("--info"), &deb]);
    for forbidden in ["postinst", "preinst", "postrm", "prerm"] {
        assert!(
            !info.contains(forbidden),
            "the package must ship no {forbidden}:\n{info}"
        );
    }
}

#[test]
fn missing_dpkg_tooling_names_the_tool_and_writes_nothing() {
    let out = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    // The child gets a PATH with no dpkg at all, so bash itself is named by its full path.
    let bash = run("sh", &["-c", "command -v bash"]);
    let run_script = Command::new(&bash)
        .arg(repo_root().join("scripts/build_deb.sh"))
        .arg("--out")
        .arg(out.path())
        .env("EASY_NOTE_BIN", "/bin/true")
        .env("PATH", empty.path())
        .current_dir(repo_root())
        .output()
        .unwrap();

    assert_eq!(
        run_script.status.code(),
        Some(1),
        "a missing tool must exit 1, not {:?}",
        run_script.status
    );
    let err = String::from_utf8_lossy(&run_script.stderr);
    assert!(
        err.contains("need dpkg"),
        "stderr must name the tool: {err:?}"
    );
    assert!(
        err.contains("sudo apt install dpkg-dev"),
        "stderr must name the package that carries it: {err:?}"
    );
    assert_eq!(
        std::fs::read_dir(out.path()).unwrap().count(),
        0,
        "a refused build must write no package"
    );
}

#[test]
fn the_release_build_takes_window_and_tray_unless_features_say_otherwise() {
    if Command::new("dpkg-deb").arg("--version").output().is_err() {
        eprintln!("skipped: dpkg-deb is not on PATH");
        return;
    }
    // A `cargo` stand-in records the build arguments, so the default feature list is checked
    // without a release build. What the script does after the build is not this test's subject:
    // it packages whatever `target/release/easy-note` holds, or stops because nothing is there.
    let shim_dir = tempfile::tempdir().unwrap();
    let log = shim_dir.path().join("cargo-args");
    std::fs::write(
        shim_dir.path().join("cargo"),
        format!("#!/bin/sh\necho \"$@\" >> {}\n", log.display()),
    )
    .unwrap();
    std::fs::set_permissions(
        shim_dir.path().join("cargo"),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .unwrap();
    let path = format!(
        "{}:{}",
        shim_dir.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    for (features, want) in [
        (None, "--features window,tray"),
        (Some("tray"), "--features tray"),
    ] {
        let out = tempfile::tempdir().unwrap();
        let mut cmd = Command::new("bash");
        cmd.arg(repo_root().join("scripts/build_deb.sh"))
            .arg("--out")
            .arg(out.path());
        if let Some(features) = features {
            cmd.arg("--features").arg(features);
        }
        // EASY_NOTE_BIN stays unset here: this is the cargo path.
        cmd.env_remove("EASY_NOTE_BIN")
            .env("PATH", &path)
            .current_dir(repo_root());
        cmd.output().unwrap();
        let args = std::fs::read_to_string(&log).unwrap();
        assert!(
            args.lines().any(|l| l.contains(want)),
            "no {want:?} in the recorded cargo arguments:\n{args}"
        );
    }
}
