use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::session::WebDriverBiDiSession;

/// Send a command to the WebDriver BiDi session and processes the result.
pub async fn send_command<C, R>(
    session: &mut WebDriverBiDiSession,
    command: C,
) -> Result<R, CommandError>
where
    C: Serialize,
    R: for<'de> Deserialize<'de>,
{
    match session.send_command::<C, R>(command).await {
        Ok(rslt) => {
            // debug!("Command with id: {} succeeded", command_id); // Log success
            Ok(rslt)
        }
        Err(e) => {
            // error!("Command with id: {} failed: {:?}", command_id, e); // Log error
            Err(e)
        }
    }
}

/// Macro to define a WebDriver BiDi command.
///
/// This macro generates a struct representing the command, an implementation block
/// to create a new instance of the command, and an asynchronous function to send
/// the command to the WebDriver BiDi session.
///
/// # Parameters
///
/// - `$cmd_name`: The name of the command struct to be generated.
/// - `$cmd_type`: The type of the command parameters.
/// - `$params_type`: The type of the parameters passed to the command.
/// - `$fn_name`: The name of the function to send the command.
/// - `$result_type`: The type of the result returned by the command.
#[macro_export]
macro_rules! define_command {
    ($cmd_name:ident, $cmd_type:ty, $params_type:ty, $fn_name:ident, $result_type:ty) => {
        #[derive(Debug, Serialize, Deserialize)]
        struct $cmd_name {
            id: u64,
            #[serde(flatten)]
            params: $cmd_type,
        }

        impl $cmd_name {
            fn new(params: $params_type) -> Self {
                let id = id::get_next_id();
                // debug!("Creating {} with id: {}", $cmd_str, id);
                let params = <$cmd_type>::new(params);
                Self { id, params }
            }
        }

        pub async fn $fn_name(
            session: &mut WebDriverBiDiSession,
            params: $params_type,
        ) -> Result<$result_type, CommandError> {
            let cmd = $cmd_name::new(params);
            utils::send_command(session, cmd).await
        }
    };
}
