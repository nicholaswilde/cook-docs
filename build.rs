use std::process::Command;

fn main() {
    // Attempt to get the version from git describe
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output();

    let git_version = match output {
        Ok(o) if o.status.success() => String::from_utf8(o.stdout).ok(),
        _ => None,
    };

    // Determine the version to use
    let version = if let Some(ver) = git_version {
        ver.trim().to_string()
    } else {
        // Fallback to Cargo.toml version if git fails (e.g., tarball build)
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string())
    };

    // Set the PROJECT_VERSION environment variable for the application to use
    println!("cargo:rustc-env=PROJECT_VERSION={}", version);

    // Ensure build.rs reruns if git HEAD changes (branch switch, commit)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
}
