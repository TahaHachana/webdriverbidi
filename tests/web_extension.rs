//! Live tests for the `webExtension` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::web_extension::*;

use support::{error_code, supported, temp_dir, with_session};

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/web_extension/install
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/web_extension/uninstall
#[tokio::test]
async fn install_and_uninstall_from_path() -> Result<()> {
    with_session(|mut session| async move {
        let dir = temp_dir("extension")?;
        std::fs::write(
            dir.join("manifest.json"),
            r#"{ "manifest_version": 3, "name": "webdriverbidi test", "version": "1.0" }"#,
        )?;
        let installed = session
            .web_extension_install(InstallParameters::new(ExtensionData::ExtensionPath(
                ExtensionPath::new(dir.to_string_lossy().into()),
            )))
            .await;
        let Some(installed) = supported("webExtension.install", installed)? else {
            return Ok(());
        };
        ensure!(!installed.extension.is_empty(), "empty extension id");
        session
            .web_extension_uninstall(UninstallParameters::new(installed.extension))
            .await?;
        let _ = std::fs::remove_dir_all(dir);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn installing_a_missing_extension_fails() -> Result<()> {
    with_session(|mut session| async move {
        let result = session
            .web_extension_install(InstallParameters::new(ExtensionData::ExtensionPath(
                ExtensionPath::new("/nonexistent/extension".into()),
            )))
            .await;
        // The spec says `invalid web extension`; Chrome 158 answers `unknown error`
        // ("File path cannot be resolved"), so only require that it fails.
        match result {
            Ok(_) => anyhow::bail!("installing a missing extension succeeded"),
            Err(e) => {
                if error_code(&e) != Some("invalid web extension") {
                    eprintln!(
                        "NOTE: the browser answered {:?}, not `invalid web extension`",
                        error_code(&e)
                    );
                }
            }
        }
        Ok(())
    })
    .await
}
