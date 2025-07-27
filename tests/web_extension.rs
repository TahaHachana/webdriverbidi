use anyhow::Result;
use webdriverbidi::{
    error::CommandError,
    model::web_extension::{ExtensionArchivePath, ExtensionData, InstallParameters},
};

mod utils;

mod install_extension {

    use super::*;

    #[tokio::test]
    async fn test_nonexistent_extension() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let err = bidi_session
            .web_extension_install(InstallParameters::new(ExtensionData::ExtensionArchivePath(
                ExtensionArchivePath::new("doesnotexist".to_owned()),
            )))
            .await
            .unwrap_err();
        assert!(matches!(err, CommandError::Error(_)));

        utils::session::close(&mut bidi_session).await?;

        Ok(())
    }
}
