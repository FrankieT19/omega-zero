use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/logs/HEAD");

    let hash = git(&["rev-parse", "--short", "HEAD"])
        .or_else(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let root = root.canonicalize().ok()?;
            git(&[
                "config",
                "--global",
                "--add",
                "safe.directory",
                root.to_str()?,
            ])?;
            git(&["rev-parse", "--short", "HEAD"])
        })
        .unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain"]).is_some_and(|s| !s.is_empty());
    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    println!("cargo:rustc-env=SLOT_GIT_HASH={hash}");
    println!("cargo:rustc-env=SLOT_GIT_DIRTY={}", u8::from(dirty));
    println!("cargo:rustc-env=SLOT_BUILD_DATE={date}");

    screenscraper_key();
}

/// The same mask `label_sync::unmask` takes off.
fn mask(text: &str) -> String {
    text.bytes()
        .enumerate()
        .map(|(i, b)| {
            format!(
                "{:02x}",
                b ^ (0x5a_u8.wrapping_add((i as u8).wrapping_mul(31)))
            )
        })
        .collect()
}

/// ScreenScraper's key for slot, from the environment of whoever builds it, masked so it is
/// not lying in the binary as plain text.
fn screenscraper_key() {
    println!("cargo:rerun-if-env-changed=SLOT_SS_DEVID");
    println!("cargo:rerun-if-env-changed=SLOT_SS_DEVPASSWORD");
    let id = std::env::var("SLOT_SS_DEVID").unwrap_or_default();
    let pw = std::env::var("SLOT_SS_DEVPASSWORD").unwrap_or_default();
    if !id.is_empty() && !pw.is_empty() {
        println!("cargo:rustc-env=SLOT_SS_KEY={}:{}", mask(&id), mask(&pw));
    }
}
