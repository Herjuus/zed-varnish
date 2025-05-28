use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree, register_extension};

const VARNISHLS_REPO: &str = "M4R7iNP/varnishls";

struct VarnishExtension {
    cached_binary_path: Option<String>,
}

impl VarnishExtension {
    fn varnishls_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<String> {
        // Binary cache for session
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let latest_release = zed::latest_github_release(
            VARNISHLS_REPO,
            zed::GithubReleaseOptions {
                pre_release: false,
                require_assets: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let filename = match (platform, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => "varnishls-x86_64-unknown-linux-gnu",
            (zed::Os::Mac, zed::Architecture::Aarch64) => "varnishls-aarch64-apple-darwin",
            (zed::Os::Mac, zed::Architecture::X8664) => "varnishls-x86_64-apple-darwin",
            (zed::Os::Windows, zed::Architecture::X8664) => "varnishls-x86_64-pc-windows-msvc.exe",
            _ => {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Failed(
                        "Varnishls not supported on this device".into(),
                    ),
                );
                return Err("Unsupported platform or architecture".into());
            }
        };

        let download_url = format!(
            "https://github.com/{repo}/releases/download/{version}/{filename}",
            repo = VARNISHLS_REPO,
            version = latest_release.version,
            filename = filename,
        );

        let version_dir = format!("varnishls-{}", latest_release.version);
        let binary_path = format!("{}/{}", version_dir, filename);

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &download_url,
                &version_dir,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for VarnishExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<zed_extension_api::Command> {
        Ok(zed::Command {
            command: self.varnishls_server_binary_path(language_server_id, worktree)?,
            args: vec!["lsp".into(), "--stdio".into()],
            env: Default::default(),
        })
    }
}

register_extension!(VarnishExtension);
