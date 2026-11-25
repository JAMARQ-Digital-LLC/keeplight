use std::path::PathBuf;

fn main() {
    // A release build from a full checkout, such as the installer's, is the
    // release app; a debug build, or any build from a git worktree, is the
    // dev app with its own folder and names (src/scratch.rs).
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets this"))
        .ancestors()
        .nth(2)
        .expect("crates/core sits two levels below the repo root")
        .to_path_buf();
    let release = std::env::var("PROFILE").as_deref() == Ok("release");
    let worktree = root.join(".git").is_file();
    let identity = if release && !worktree { "release" } else { "dev" };
    println!("cargo:rustc-env=KEEPLIGHT_IDENTITY={identity}");
    println!("cargo:rerun-if-changed=build.rs");
}
