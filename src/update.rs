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
    run_self_update_with(
        &opts,
        &target,
        std::io::stdin().is_terminal(),
        out_writer,
        |version| fetch_target_release(&lookup_updater, version),
        download_checksum_asset,
        |checksum| {
            let updater = build_updater(&opts, &target, checksum)?;
            Ok(updater.update_extended())
        },
    )
}

fn run_self_update_with<FR, FC, FU>(
    opts: &SelfUpdateOptions,
    target: &str,
    is_interactive: bool,
    out_writer: &mut dyn Write,
    fetch_release: FR,
    fetch_checksum: FC,
    perform_update: FU,
) -> Result<(), String>
where
    FR: FnOnce(Option<&str>) -> Result<Release, String>,
    FC: FnOnce(&ReleaseAsset, &str) -> Result<Checksum, String>,
    FU: FnOnce(
        Option<Checksum>,
    ) -> Result<Result<ReleaseStatus, self_update::errors::Error>, String>,
{
    if opts.check {
        let release = fetch_release(opts.version.as_deref())?;
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

        match select_release_asset(release.assets(), target) {
            Some(asset) => writeln!(out_writer, "Asset: {}", asset.name()),
            None => writeln!(
                out_writer,
                "Warning: no release asset found for target '{target}'."
            ),
        }
        .map_err(|e| format!("Failed writing output: {e}"))?;
        return Ok(());
    }

    if !opts.yes && !is_interactive {
        return Err(
            "Cannot confirm update in non-interactive mode; pass --yes (-y) to skip confirmation."
                .to_string(),
        );
    }

    let release = fetch_release(opts.version.as_deref())?;
    if opts.version.is_none() && !is_newer_version(CURRENT_VERSION, release.version())? {
        writeln!(
            out_writer,
            "cook-docs is already up to date (v{CURRENT_VERSION})."
        )
        .map_err(|e| format!("Failed writing output: {e}"))?;
        return Ok(());
    }

    let archive_asset = select_release_asset(release.assets(), target).ok_or_else(|| {
        format!(
            "No release asset found for target '{target}' in release v{}",
            release.version()
        )
    })?;
    let checksum = select_checksum_asset(release.assets(), archive_asset.name())
        .map(|sha_asset| fetch_checksum(&sha_asset, archive_asset.name()))
        .transpose()?;

    match perform_update(checksum)? {
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

    fn serve_http_once(
        status: &'static str,
        body: Vec<u8>,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let header = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        });
        (format!("http://{addr}"), handle)
    }

    fn mock_release(version: &str, assets: Vec<ReleaseAsset>) -> Release {
        let mut builder = Release::builder();
        builder.version(version).name(format!("v{version}"));
        for asset in assets {
            builder.asset(asset);
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_bin_path_fallback_and_invalid_version_error() {
        assert_eq!(
            bin_path_in_archive_for_target("riscv64gc-unknown-linux-gnu"),
            "{{ bin }}"
        );
        let err = is_newer_version("not-semver", "1.0.0").unwrap_err();
        assert!(err.contains("Failed to compare versions"));
    }

    #[test]
    fn test_download_checksum_asset_mock_http() {
        let hash = "9a49cefe852b30cf9e9b309a7d6f24afe6406e7e7e7f6205722e95b4f4742176";
        let archive = "cook-docs_9.9.9_Linux_x86_64.tar.gz";

        // 1. Valid checksum download
        let (base, h1) = serve_http_once("200 OK", format!("{hash}  {archive}\n").into_bytes());
        let asset = ReleaseAsset::new(
            format!("{archive}.sha256"),
            format!("{base}/{archive}.sha256"),
        );
        let sum = download_checksum_asset(&asset, archive).unwrap();
        assert!(matches!(sum, Checksum::Sha256(ref s) if s == hash));
        h1.join().unwrap();

        // 2. Non-UTF8 body returns error
        let (base2, h2) = serve_http_once("200 OK", vec![0xff, 0xfe, 0xfd]);
        let bad_utf8 = ReleaseAsset::new("checksums.txt", format!("{base2}/checksums.txt"));
        let err_utf8 = download_checksum_asset(&bad_utf8, archive).unwrap_err();
        assert!(err_utf8.contains("not valid UTF-8"));
        h2.join().unwrap();

        // 3. HTTP 404 returns download error
        let (base3, h3) = serve_http_once("404 Not Found", b"not found".to_vec());
        let missing = ReleaseAsset::new("checksums.txt", format!("{base3}/checksums.txt"));
        let err_dl = download_checksum_asset(&missing, archive).unwrap_err();
        assert!(err_dl.contains("Failed to download checksum asset"));
        h3.join().unwrap();
    }

    #[test]
    fn test_fetch_target_release_mock_http() {
        let release_json = r#"{
            "name": "v9.9.9",
            "tag_name": "v9.9.9",
            "created_at": "2026-01-01T00:00:00Z",
            "published_at": "2026-01-01T00:00:00Z",
            "body": "test release",
            "html_url": "https://github.com/nicholaswilde/cook-docs/releases/tag/v9.9.9",
            "assets": [
                {
                    "name": "cook-docs_9.9.9_Linux_x86_64.tar.gz",
                    "url": "http://127.0.0.1:1/asset",
                    "browser_download_url": "http://127.0.0.1:1/asset"
                }
            ]
        }"#;

        // 1. Latest release success
        let (base1, h1) = serve_http_once("200 OK", release_json.as_bytes().to_vec());
        let updater1 = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .current_version(CURRENT_VERSION)
            .api_base_url(&base1)
            .build()
            .unwrap();
        let rel = fetch_target_release(&updater1, None).unwrap();
        assert_eq!(rel.version(), "9.9.9");
        h1.join().unwrap();

        // 2. Specific tag release success
        let (base2, h2) = serve_http_once("200 OK", release_json.as_bytes().to_vec());
        let updater2 = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .current_version(CURRENT_VERSION)
            .api_base_url(&base2)
            .build()
            .unwrap();
        let rel_tag = fetch_target_release(&updater2, Some("9.9.9")).unwrap();
        assert_eq!(rel_tag.version(), "9.9.9");
        h2.join().unwrap();

        // 3. HTTP 404 error for both latest and specific tag
        let (base3, h3) = serve_http_once("404 Not Found", b"{}".to_vec());
        let updater3 = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .current_version(CURRENT_VERSION)
            .api_base_url(&base3)
            .build()
            .unwrap();
        let err_latest = fetch_target_release(&updater3, None).unwrap_err();
        assert!(err_latest.contains("Failed to fetch latest release"));
        h3.join().unwrap();

        let (base4, h4) = serve_http_once("404 Not Found", b"{}".to_vec());
        let updater4 = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(BIN_NAME)
            .current_version(CURRENT_VERSION)
            .api_base_url(&base4)
            .build()
            .unwrap();
        let err_tag = fetch_target_release(&updater4, Some("9.9.9")).unwrap_err();
        assert!(err_tag.contains("Failed to fetch release 9.9.9"));
        h4.join().unwrap();
    }

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("write failed"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn run_update_case(
        opts: SelfUpdateOptions,
        is_interactive: bool,
        writer: &mut dyn Write,
        release: Release,
        checksum: Result<Checksum, String>,
        outcome: Result<ReleaseStatus, self_update::errors::Error>,
    ) -> Result<(), String> {
        run_self_update_with(
            &opts,
            "x86_64-unknown-linux-musl",
            is_interactive,
            writer,
            |_| Ok(release),
            |_, _| checksum,
            |_| Ok(outcome),
        )
    }

    #[test]
    fn test_run_self_update_with_check_and_install_branches() {
        let linux_asset = mock_asset("cook-docs_99.0.0_Linux_x86_64.tar.gz");
        let sha_asset = mock_asset("cook-docs_99.0.0_Linux_x86_64.tar.gz.sha256");
        let expected_sum = Checksum::Sha256("a".repeat(64));
        let newer = mock_release("99.0.0", vec![linux_asset.clone(), sha_asset.clone()]);
        let current = mock_release(CURRENT_VERSION, vec![]);
        let older = mock_release("0.1.0", vec![]);

        // 1. --check: newer version with matching asset
        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: true,
                yes: false,
                version: None,
            },
            false,
            &mut out,
            mock_release("99.0.0", vec![linux_asset.clone()]),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("Update available:"));
        assert!(s.contains("Asset: cook-docs_99.0.0_Linux_x86_64.tar.gz"));

        // 2. --check: current version with missing asset
        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: true,
                yes: false,
                version: None,
            },
            false,
            &mut out,
            current.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("cook-docs is up to date"));
        assert!(s.contains("Warning: no release asset found"));

        // 3. --check: older release found
        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: true,
                yes: false,
                version: Some("0.1.0".to_string()),
            },
            false,
            &mut out,
            older.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("Release v0.1.0 found")
        );

        // 4. Non-interactive guard without --yes
        let mut out = Vec::new();
        let err = run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: false,
                version: None,
            },
            false,
            &mut out,
            current.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap_err();
        assert!(err.contains("--yes"));

        // 5. Already up to date (no version override)
        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            current.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("already up to date")
        );

        // 6. Newer release missing asset for target
        let mut out = Vec::new();
        let err_asset = run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            mock_release("99.0.0", vec![]),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap_err();
        assert!(err_asset.contains("No release asset found"));

        // 7. Checksum fetch failure propagates
        let mut out = Vec::new();
        let err_sum = run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            newer.clone(),
            Err("checksum download failed".to_string()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap_err();
        assert!(err_sum.contains("checksum download failed"));

        // 8. Full update outcomes: Updated, UpToDate, Aborted, Error
        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            newer.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::Updated(newer.clone())),
        )
        .unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("Successfully updated cook-docs")
        );

        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            newer.clone(),
            Ok(expected_sum.clone()),
            Ok(ReleaseStatus::UpToDate),
        )
        .unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("already up to date")
        );

        let mut out = Vec::new();
        run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: false,
                version: None,
            },
            true,
            &mut out,
            newer.clone(),
            Ok(expected_sum.clone()),
            Err(self_update::errors::Error::Aborted),
        )
        .unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("Update cancelled.")
        );

        let mut out = Vec::new();
        let err_upd = run_update_case(
            SelfUpdateOptions {
                check: false,
                yes: true,
                version: None,
            },
            false,
            &mut out,
            newer.clone(),
            Ok(expected_sum.clone()),
            Err(self_update::errors::Error::no_release_found()),
        )
        .unwrap_err();
        assert!(err_upd.contains("Failed to self-update cook-docs"));

        // 9. Writer failures return formatted output errors
        for (opts, rel, outcome) in [
            (
                SelfUpdateOptions {
                    check: true,
                    yes: false,
                    version: None,
                },
                newer.clone(),
                Ok(ReleaseStatus::UpToDate),
            ),
            (
                SelfUpdateOptions {
                    check: true,
                    yes: false,
                    version: None,
                },
                current.clone(),
                Ok(ReleaseStatus::UpToDate),
            ),
            (
                SelfUpdateOptions {
                    check: true,
                    yes: false,
                    version: Some("0.1.0".to_string()),
                },
                older,
                Ok(ReleaseStatus::UpToDate),
            ),
            (
                SelfUpdateOptions {
                    check: false,
                    yes: true,
                    version: None,
                },
                current,
                Ok(ReleaseStatus::UpToDate),
            ),
            (
                SelfUpdateOptions {
                    check: false,
                    yes: true,
                    version: None,
                },
                newer.clone(),
                Ok(ReleaseStatus::UpToDate),
            ),
            (
                SelfUpdateOptions {
                    check: false,
                    yes: true,
                    version: None,
                },
                newer.clone(),
                Ok(ReleaseStatus::Updated(newer.clone())),
            ),
            (
                SelfUpdateOptions {
                    check: false,
                    yes: true,
                    version: None,
                },
                newer,
                Err(self_update::errors::Error::Aborted),
            ),
        ] {
            let err = run_update_case(
                opts,
                false,
                &mut FailingWriter,
                rel,
                Ok(expected_sum.clone()),
                outcome,
            )
            .unwrap_err();
            assert!(err.contains("Failed writing output"));
        }
    }
}
