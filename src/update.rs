use std::io::{IsTerminal, Write};

use self_update::Checksum;
use self_update::update::ReleaseAsset;
use self_update::{Release, ReleaseStatus, UpdateStrategy};

pub const REPO_OWNER: &str = "nicholaswilde";
pub const REPO_NAME: &str = "cook-docs";
pub const BIN_NAME: &str = "cook-docs";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Options for the `cook-docs self-update` command.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelfUpdateOptions {
    pub check: bool,
    pub yes: bool,
    pub version: Option<String>,
}

/// Map the host target triple to the corresponding release target built by CI.
pub fn resolve_target(raw_target: &str) -> &str {
    match raw_target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64-unknown-linux-musl",
        "armv7-unknown-linux-gnueabihf" | "armv7-unknown-linux-musleabihf" => {
            "armv7-unknown-linux-musleabihf"
        }
        "arm-unknown-linux-gnueabihf"
        | "arm-unknown-linux-gnueabi"
        | "arm-unknown-linux-musleabihf"
        | "arm-unknown-linux-musleabi" => "arm-unknown-linux-gnueabihf",
        "x86_64-apple-darwin" => "x86_64-apple-darwin",
        "aarch64-apple-darwin" => "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc" | "aarch64-pc-windows-gnu" => "aarch64-pc-windows-msvc",
        other => other,
    }
}

/// Map a target triple to the `<os>_<arch>` label used in `cook-docs` release asset names.
pub fn release_platform_for_target(target: &str) -> Option<&'static str> {
    match resolve_target(target) {
        "x86_64-unknown-linux-musl" => Some("Linux_x86_64"),
        "aarch64-unknown-linux-musl" => Some("Linux_arm64"),
        "armv7-unknown-linux-musleabihf" => Some("Linux_armv7"),
        "arm-unknown-linux-gnueabihf" => Some("Linux_armv6"),
        "x86_64-apple-darwin" => Some("Darwin_x86_64"),
        "aarch64-apple-darwin" => Some("Darwin_arm64"),
        "x86_64-pc-windows-msvc" => Some("Windows_x86_64"),
        "aarch64-pc-windows-msvc" => Some("Windows_arm64"),
        _ => None,
    }
}

/// Return the release archive extension for the given target triple.
pub fn archive_extension_for_target(target: &str) -> &'static str {
    if target.contains("windows") {
        ".zip"
    } else {
        ".tar.gz"
    }
}

/// Build the archive-relative binary path template for `self_update`.
pub fn bin_path_in_archive_for_target(target: &str) -> String {
    match release_platform_for_target(target) {
        Some(platform) => format!("cook-docs_{{{{ version }}}}_{platform}/{{{{ bin }}}}"),
        None => "{{ bin }}".to_string(),
    }
}

/// Select the matching `cook-docs_<ver>_<os>_<arch>` archive asset, ignoring `.sha256` and signature files.
pub fn select_release_asset(assets: &[ReleaseAsset], target: &str) -> Option<ReleaseAsset> {
    let platform = release_platform_for_target(target)?;
    let ext = archive_extension_for_target(target);
    let suffix = format!("_{platform}{ext}");
    assets
        .iter()
        .find(|asset| asset.name().starts_with("cook-docs_") && asset.name().ends_with(&suffix))
        .cloned()
}

/// Select the `.sha256` sidecar asset for `archive_asset_name`, falling back to `checksums.txt`.
pub fn select_checksum_asset(
    assets: &[ReleaseAsset],
    archive_asset_name: &str,
) -> Option<ReleaseAsset> {
    let sidecar = format!("{archive_asset_name}.sha256");
    assets
        .iter()
        .find(|asset| asset.name() == sidecar)
        .or_else(|| assets.iter().find(|asset| asset.name() == "checksums.txt"))
        .cloned()
}

/// Parse a SHA-256 checksum from a `.sha256` sidecar (GNU coreutils, BSD, bare hex, or Windows `certutil`) or `checksums.txt`.
pub fn parse_sha256_checksum(contents: &str, archive_name: &str) -> Result<Checksum, String> {
    if let Ok(checksum) = Checksum::from_sums_file(contents, archive_name) {
        return Ok(checksum);
    }

    // Windows CI generates `.zip.sha256` with `certutil -hashfile <file> SHA256`.
    if contents.contains("SHA256 hash of")
        && let Some(hex) = contents
            .lines()
            .map(str::trim)
            .find(|line| line.len() == 64 && line.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Ok(Checksum::Sha256(hex.to_ascii_lowercase()));
    }

    Err(format!(
        "Failed to parse SHA-256 checksum for '{archive_name}'"
    ))
}

/// Normalize a user-supplied version or tag string to a `v`-prefixed GitHub tag.
pub fn normalize_version_tag(version: &str) -> String {
    let trimmed = version.trim();
    if let Some(rest) = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
    {
        format!("v{rest}")
    } else {
        format!("v{trimmed}")
    }
}

fn strip_v_prefix(version: &str) -> &str {
    let trimmed = version.trim();
    trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed)
}

/// Return `true` if `candidate` is strictly newer than `current` according to SemVer.
pub fn is_newer_version(current: &str, candidate: &str) -> Result<bool, String> {
    self_update::version::bump_is_greater(strip_v_prefix(current), strip_v_prefix(candidate))
        .map_err(|e| format!("Failed to compare versions '{current}' and '{candidate}': {e}"))
}

fn build_updater(
    opts: &SelfUpdateOptions,
    target: &str,
    checksum: Option<Checksum>,
) -> Result<self_update::backends::github::Update, String> {
    let matcher_target = target.to_string();
    let bin_path = bin_path_in_archive_for_target(target);
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(CURRENT_VERSION)
        .target(target)
        .bin_path_in_archive(&bin_path)
        .asset_matcher(move |assets| select_release_asset(assets, &matcher_target))
        .update_strategy(UpdateStrategy::Latest)
        .show_download_progress(!opts.check)
        .show_output(!opts.check)
        .no_confirm(opts.yes)
        .check_install_path_writable(!opts.check)
        .auth_token_from_env();

    if let Some(ref ver) = opts.version {
        builder.release_tag(normalize_version_tag(ver));
    }
    if let Some(sum) = checksum {
        builder.verify_checksum(sum);
    }

    builder
        .build()
        .map_err(|e| format!("Failed to configure updater: {e}"))
}

fn fetch_target_release(
    updater: &self_update::backends::github::Update,
    version: Option<&str>,
) -> Result<Release, String> {
    if let Some(ver) = version {
        updater
            .get_release_version(&normalize_version_tag(ver))
            .map_err(|e| format!("Failed to fetch release {ver}: {e}"))
    } else {
        updater
            .get_latest_release()
            .map_err(|e| format!("Failed to fetch latest release: {e}"))?
            .into_vec()
            .into_iter()
            .next()
            .ok_or_else(|| format!("No releases found for {REPO_OWNER}/{REPO_NAME}"))
    }
}

fn download_checksum_asset(asset: &ReleaseAsset, archive_name: &str) -> Result<Checksum, String> {
    let mut download = self_update::Download::from_url(asset.download_url());
    download
        .request_header("accept", "application/octet-stream")
        .request_header("user-agent", BIN_NAME);

    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.trim().is_empty()
        && asset.download_url().starts_with("https://api.github.com/")
    {
        download.request_header("authorization", format!("Bearer {}", token.trim()));
    }

    let mut bytes = Vec::new();
    download
        .download_to(&mut bytes)
        .map_err(|e| format!("Failed to download checksum asset '{}': {e}", asset.name()))?;

    let text = String::from_utf8(bytes)
        .map_err(|e| format!("Checksum asset '{}' is not valid UTF-8: {e}", asset.name()))?;
    parse_sha256_checksum(&text, archive_name)
}

/// Check for or install a `cook-docs` release from GitHub.
pub fn run_self_update(opts: SelfUpdateOptions, out_writer: &mut dyn Write) -> Result<(), String> {
    let target = resolve_target(self_update::get_target()).to_string();
    let lookup_updater = build_updater(&opts, &target, None)?;

    if opts.check {
        let release = fetch_target_release(&lookup_updater, opts.version.as_deref())?;
        let latest_ver = release.version();
        if is_newer_version(CURRENT_VERSION, latest_ver)? {
            writeln!(
                out_writer,
                "Update available: v{CURRENT_VERSION} -> v{latest_ver}"
            )
            .map_err(|e| format!("Failed writing output: {e}"))?;
        } else if strip_v_prefix(latest_ver) == strip_v_prefix(CURRENT_VERSION) {
            writeln!(out_writer, "cook-docs is up to date (v{CURRENT_VERSION}).")
                .map_err(|e| format!("Failed writing output: {e}"))?;
        } else {
            writeln!(
                out_writer,
                "Release v{latest_ver} found (current: v{CURRENT_VERSION})."
            )
            .map_err(|e| format!("Failed writing output: {e}"))?;
        }

        match select_release_asset(release.assets(), &target) {
            Some(asset) => writeln!(out_writer, "Asset: {}", asset.name()),
            None => writeln!(
                out_writer,
                "Warning: no release asset found for target '{target}'."
            ),
        }
        .map_err(|e| format!("Failed writing output: {e}"))?;
        return Ok(());
    }

    if !opts.yes && !std::io::stdin().is_terminal() {
        return Err(
            "Cannot confirm update in non-interactive mode; pass --yes (-y) to skip confirmation."
                .to_string(),
        );
    }

    let release = fetch_target_release(&lookup_updater, opts.version.as_deref())?;
    if opts.version.is_none() && !is_newer_version(CURRENT_VERSION, release.version())? {
        writeln!(
            out_writer,
            "cook-docs is already up to date (v{CURRENT_VERSION})."
        )
        .map_err(|e| format!("Failed writing output: {e}"))?;
        return Ok(());
    }

    let archive_asset = select_release_asset(release.assets(), &target).ok_or_else(|| {
        format!(
            "No release asset found for target '{target}' in release v{}",
            release.version()
        )
    })?;
    let checksum = select_checksum_asset(release.assets(), archive_asset.name())
        .map(|sha_asset| download_checksum_asset(&sha_asset, archive_asset.name()))
        .transpose()?;

    let updater = build_updater(&opts, &target, checksum)?;
    match updater.update_extended() {
        Ok(ReleaseStatus::UpToDate) => {
            writeln!(
                out_writer,
                "cook-docs is already up to date (v{CURRENT_VERSION})."
            )
            .map_err(|e| format!("Failed writing output: {e}"))?;
        }
        Ok(ReleaseStatus::Updated(release)) => {
            writeln!(
                out_writer,
                "Successfully updated cook-docs from v{CURRENT_VERSION} to v{}.",
                release.version()
            )
            .map_err(|e| format!("Failed writing output: {e}"))?;
        }
        Ok(other) => {
            writeln!(
                out_writer,
                "Update status: v{}.",
                other
                    .into_version_status(CURRENT_VERSION.to_string())
                    .version()
            )
            .map_err(|e| format!("Failed writing output: {e}"))?;
        }
        Err(self_update::errors::Error::Aborted) => {
            writeln!(out_writer, "Update cancelled.")
                .map_err(|e| format!("Failed writing output: {e}"))?;
        }
        Err(err) => {
            return Err(format!("Failed to self-update cook-docs: {err}"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_asset(name: &str) -> ReleaseAsset {
        ReleaseAsset::new(name, format!("https://example.com/{name}"))
    }

    #[test]
    fn test_resolve_target_mappings() {
        assert_eq!(
            resolve_target("x86_64-unknown-linux-gnu"),
            "x86_64-unknown-linux-musl"
        );
        assert_eq!(
            resolve_target("x86_64-unknown-linux-musl"),
            "x86_64-unknown-linux-musl"
        );
        assert_eq!(
            resolve_target("aarch64-unknown-linux-gnu"),
            "aarch64-unknown-linux-musl"
        );
        assert_eq!(
            resolve_target("aarch64-unknown-linux-musl"),
            "aarch64-unknown-linux-musl"
        );
        assert_eq!(
            resolve_target("armv7-unknown-linux-gnueabihf"),
            "armv7-unknown-linux-musleabihf"
        );
        assert_eq!(
            resolve_target("armv7-unknown-linux-musleabihf"),
            "armv7-unknown-linux-musleabihf"
        );
        assert_eq!(
            resolve_target("arm-unknown-linux-gnueabihf"),
            "arm-unknown-linux-gnueabihf"
        );
        assert_eq!(
            resolve_target("arm-unknown-linux-musleabi"),
            "arm-unknown-linux-gnueabihf"
        );
        assert_eq!(resolve_target("x86_64-apple-darwin"), "x86_64-apple-darwin");
        assert_eq!(
            resolve_target("aarch64-apple-darwin"),
            "aarch64-apple-darwin"
        );
        assert_eq!(
            resolve_target("x86_64-pc-windows-msvc"),
            "x86_64-pc-windows-msvc"
        );
        assert_eq!(
            resolve_target("x86_64-pc-windows-gnu"),
            "x86_64-pc-windows-msvc"
        );
        assert_eq!(
            resolve_target("aarch64-pc-windows-msvc"),
            "aarch64-pc-windows-msvc"
        );
        assert_eq!(
            resolve_target("aarch64-pc-windows-gnu"),
            "aarch64-pc-windows-msvc"
        );
        assert_eq!(
            resolve_target("riscv64gc-unknown-linux-gnu"),
            "riscv64gc-unknown-linux-gnu"
        );
    }

    #[test]
    fn test_release_platform_and_archive_extension() {
        assert_eq!(
            release_platform_for_target("x86_64-unknown-linux-gnu"),
            Some("Linux_x86_64")
        );
        assert_eq!(
            release_platform_for_target("aarch64-unknown-linux-musl"),
            Some("Linux_arm64")
        );
        assert_eq!(
            release_platform_for_target("armv7-unknown-linux-musleabihf"),
            Some("Linux_armv7")
        );
        assert_eq!(
            release_platform_for_target("arm-unknown-linux-gnueabihf"),
            Some("Linux_armv6")
        );
        assert_eq!(
            release_platform_for_target("x86_64-apple-darwin"),
            Some("Darwin_x86_64")
        );
        assert_eq!(
            release_platform_for_target("aarch64-apple-darwin"),
            Some("Darwin_arm64")
        );
        assert_eq!(
            release_platform_for_target("x86_64-pc-windows-msvc"),
            Some("Windows_x86_64")
        );
        assert_eq!(
            release_platform_for_target("aarch64-pc-windows-msvc"),
            Some("Windows_arm64")
        );
        assert_eq!(
            release_platform_for_target("riscv64gc-unknown-linux-gnu"),
            None
        );

        assert_eq!(
            archive_extension_for_target("x86_64-unknown-linux-musl"),
            ".tar.gz"
        );
        assert_eq!(
            archive_extension_for_target("aarch64-apple-darwin"),
            ".tar.gz"
        );
        assert_eq!(
            archive_extension_for_target("x86_64-pc-windows-msvc"),
            ".zip"
        );
        assert_eq!(
            archive_extension_for_target("aarch64-pc-windows-msvc"),
            ".zip"
        );
        assert_eq!(
            bin_path_in_archive_for_target("x86_64-unknown-linux-musl"),
            "cook-docs_{{ version }}_Linux_x86_64/{{ bin }}"
        );
    }

    #[test]
    fn test_select_release_and_checksum_assets() {
        let assets = vec![
            mock_asset("cook-docs_0.9.1_Linux_x86_64.tar.gz.sha256"),
            mock_asset("cook-docs_0.9.1_Linux_x86_64.tar.gz"),
            mock_asset("cook-docs_0.9.1_Linux_arm64.tar.gz"),
            mock_asset("cook-docs_0.9.1_Linux_armv7.tar.gz"),
            mock_asset("cook-docs_0.9.1_Linux_armv6.tar.gz"),
            mock_asset("cook-docs_0.9.1_Darwin_x86_64.tar.gz"),
            mock_asset("cook-docs_0.9.1_Windows_x86_64.zip.sha256"),
            mock_asset("cook-docs_0.9.1_Windows_x86_64.zip"),
            mock_asset("checksums.txt"),
            mock_asset("checksums.txt.sig"),
        ];

        let linux = select_release_asset(&assets, "x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(linux.name(), "cook-docs_0.9.1_Linux_x86_64.tar.gz");
        assert_eq!(
            select_checksum_asset(&assets, linux.name()).unwrap().name(),
            "cook-docs_0.9.1_Linux_x86_64.tar.gz.sha256"
        );

        let arm64 = select_release_asset(&assets, "aarch64-unknown-linux-musl").unwrap();
        assert_eq!(arm64.name(), "cook-docs_0.9.1_Linux_arm64.tar.gz");
        assert_eq!(
            select_checksum_asset(&assets, arm64.name()).unwrap().name(),
            "checksums.txt"
        );

        let win = select_release_asset(&assets, "x86_64-pc-windows-msvc").unwrap();
        assert_eq!(win.name(), "cook-docs_0.9.1_Windows_x86_64.zip");
        assert_eq!(
            select_checksum_asset(&assets, win.name()).unwrap().name(),
            "cook-docs_0.9.1_Windows_x86_64.zip.sha256"
        );

        assert!(select_release_asset(&assets, "aarch64-apple-darwin").is_none());
    }

    #[test]
    fn test_parse_sha256_checksum_formats() {
        let linux_sum = "9a49cefe852b30cf9e9b309a7d6f24afe6406e7e7e7f6205722e95b4f4742176  cook-docs_0.9.1_Linux_x86_64.tar.gz\n";
        assert!(matches!(
            parse_sha256_checksum(linux_sum, "cook-docs_0.9.1_Linux_x86_64.tar.gz").unwrap(),
            Checksum::Sha256(ref h) if h == "9a49cefe852b30cf9e9b309a7d6f24afe6406e7e7e7f6205722e95b4f4742176"
        ));

        let bare = "9a49cefe852b30cf9e9b309a7d6f24afe6406e7e7e7f6205722e95b4f4742176\n";
        assert!(matches!(
            parse_sha256_checksum(bare, "cook-docs_0.9.1_Linux_x86_64.tar.gz").unwrap(),
            Checksum::Sha256(_)
        ));

        let win_certutil = "\
SHA256 hash of cook-docs_0.9.1_Windows_x86_64.zip:\r\n\
df773797ed004ebbc39ef6360bd4776360e626e10e0a8b1c0be1931d47f705f1\r\n\
CertUtil: -hashfile command completed successfully.\r\n";
        assert!(matches!(
            parse_sha256_checksum(win_certutil, "cook-docs_0.9.1_Windows_x86_64.zip").unwrap(),
            Checksum::Sha256(ref h) if h == "df773797ed004ebbc39ef6360bd4776360e626e10e0a8b1c0be1931d47f705f1"
        ));

        assert!(parse_sha256_checksum("not a checksum", "cook-docs.tar.gz").is_err());
    }

    #[test]
    fn test_normalize_version_tag() {
        assert_eq!(normalize_version_tag("0.9.1"), "v0.9.1");
        assert_eq!(normalize_version_tag("v0.9.1"), "v0.9.1");
        assert_eq!(normalize_version_tag("V0.10.0"), "v0.10.0");
        assert_eq!(normalize_version_tag("  0.10.0  "), "v0.10.0");
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.9.1", "0.9.2").unwrap());
        assert!(is_newer_version("0.9.1", "0.10.0").unwrap());
        assert!(is_newer_version("0.9.1", "1.0.0").unwrap());
        assert!(is_newer_version("v0.9.1", "v0.9.2").unwrap());
        assert!(!is_newer_version("0.9.1", "0.9.1").unwrap());
        assert!(!is_newer_version("0.9.1", "0.9.0").unwrap());
        assert!(!is_newer_version("0.10.0", "0.10.0-rc.1").unwrap());
        assert!(is_newer_version("0.10.0-rc.1", "0.10.0").unwrap());
    }

    #[test]
    fn test_build_updater_succeeds() {
        let opts = SelfUpdateOptions {
            check: true,
            yes: false,
            version: Some("0.9.1".to_string()),
        };
        let target = resolve_target(self_update::get_target());
        assert!(build_updater(&opts, target, None).is_ok());

        let sum = Checksum::Sha256(
            "9a49cefe852b30cf9e9b309a7d6f24afe6406e7e7e7f6205722e95b4f4742176".to_string(),
        );
        assert!(build_updater(&opts, target, Some(sum)).is_ok());
    }
}
