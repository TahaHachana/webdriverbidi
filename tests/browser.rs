use std::sync::Arc;

use anyhow::Result;
use log::debug;
use tokio::sync::Mutex;
use webdriverbidi::events::EventType;
use webdriverbidi::model::browser::RemoveUserContextParameters;
use webdriverbidi::model::browsing_context::{
    ActivateParameters, CloseParameters, GetTreeParameters,
};
use webdriverbidi::model::session::SubscriptionRequest;

mod utils;

mod install_extension {
    use webdriverbidi::{
        error::CommandError,
        model::web_extension::{ExtensionArchivePath, ExtensionData, InstallParameters},
    };

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

const DEFAULT_USER_CONTEXT: &str = "default";

// --------------------------------------------------

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/create_user_context
mod create_user_context {
    use super::*;

    const TEST_STORAGE_ISOLATION_HTML: &str = "test_storage_isolation.html";

    #[tokio::test]
    async fn test_create_context() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let user_context = utils::browser::create_user_context(&mut bidi_session).await?;
        let ids = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert!(ids.contains(&user_context));

        Ok(())
    }

    #[tokio::test]
    async fn test_unique_id() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let first_context = utils::browser::create_user_context(&mut bidi_session).await?;
        let second_context = utils::browser::create_user_context(&mut bidi_session).await?;
        let ids = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert!(ids.contains(&first_context));
        assert!(ids.contains(&second_context));
        assert_ne!(first_context, second_context);

        Ok(())
    }

    #[tokio::test]
    async fn test_storage_isolation() -> Result<()> {
        let (url, server_handle) =
            utils::axum_utils::serve_static(TEST_STORAGE_ISOLATION_HTML).await?;

        let mut bidi_session = utils::session::init().await?;

        let first_context = utils::browser::create_user_context(&mut bidi_session).await?;
        let second_context = utils::browser::create_user_context(&mut bidi_session).await?;

        let tab_first_context =
            utils::browsing_context::new_tab_in_user_context(&mut bidi_session, first_context)
                .await?;
        let tab_second_context =
            utils::browsing_context::new_tab_in_user_context(&mut bidi_session, second_context)
                .await?;

        utils::browsing_context::navigate(
            &mut bidi_session,
            tab_first_context.clone(),
            url.clone(),
        )
        .await?;

        utils::browsing_context::navigate(&mut bidi_session, tab_second_context.clone(), url)
            .await?;

        let test_key = "test";
        let test_value = "value";

        let initial_tab_first_context_storage =
            utils::local_storage::get(&mut bidi_session, tab_first_context.as_str(), test_key)
                .await?;

        let initial_tab_second_context_storage =
            utils::local_storage::get(&mut bidi_session, tab_second_context.as_str(), test_key)
                .await?;

        utils::local_storage::set(
            &mut bidi_session,
            tab_first_context.as_str(),
            test_key,
            test_value,
        )
        .await?;

        let final_tab_first_context_storage =
            utils::local_storage::get(&mut bidi_session, tab_first_context.as_str(), test_key)
                .await?;

        let final_tab_second_context_storage =
            utils::local_storage::get(&mut bidi_session, tab_second_context.as_str(), test_key)
                .await?;

        utils::session::close(&mut bidi_session).await?;
        server_handle.abort();

        assert_eq!(initial_tab_first_context_storage, None);
        assert_eq!(initial_tab_second_context_storage, None);
        assert_eq!(
            final_tab_first_context_storage,
            Some(test_value.to_string())
        );
        assert_eq!(final_tab_second_context_storage, None);

        Ok(())
    }
}

// --------------------------------------------------

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/get_client_windows
mod get_client_windows {
    use super::*;

    #[tokio::test]
    async fn test_open_and_close() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let initial_windows = utils::browser::get_client_windows(&mut bidi_session).await?;
        let new_browsing_context = utils::browsing_context::new_window(&mut bidi_session).await?;
        let updated_windows = utils::browser::get_client_windows(&mut bidi_session).await?;

        bidi_session
            .browsing_context_close(CloseParameters::new(new_browsing_context, None))
            .await?;

        let final_windows = utils::browser::get_client_windows(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert_eq!(initial_windows.len(), 1);
        assert_eq!(updated_windows.len(), 2);
        assert_ne!(
            updated_windows[0].client_window,
            updated_windows[1].client_window
        );
        assert_eq!(final_windows, initial_windows);

        Ok(())
    }

    #[tokio::test]
    async fn test_activate_client_windows() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let initial_windows = utils::browser::get_client_windows(&mut bidi_session).await?;
        let initial_window = &initial_windows[0];
        let initial_window_id = &initial_window.client_window;

        let initial_contexts = bidi_session
            .browsing_context_get_tree(GetTreeParameters::new(None, None))
            .await?
            .contexts;
        let initial_context_id = &initial_contexts[0].context;

        let new_browsing_context = utils::browsing_context::new_window(&mut bidi_session).await?;

        let initial_all_windows = utils::browser::get_client_windows(&mut bidi_session).await?;
        let initial_first_window = initial_all_windows
            .iter()
            .find(|&window| window.client_window == *initial_window_id)
            .ok_or_else(|| anyhow::anyhow!("initial first window not found"))?;
        let initial_second_window = initial_all_windows
            .iter()
            .find(|&window| window.client_window != *initial_window_id)
            .ok_or_else(|| anyhow::anyhow!("initial second window not found"))?;

        bidi_session
            .browsing_context_activate(ActivateParameters::new(initial_context_id.to_string()))
            .await?;

        let final_all_windows = utils::browser::get_client_windows(&mut bidi_session).await?;
        let final_first_window = final_all_windows
            .iter()
            .find(|&window| window.client_window == *initial_window_id)
            .ok_or_else(|| anyhow::anyhow!("final first window not found"))?;
        let final_second_window = final_all_windows
            .iter()
            .find(|&window| window.client_window != *initial_window_id)
            .ok_or_else(|| anyhow::anyhow!("final second window not found"))?;

        bidi_session
            .browsing_context_close(CloseParameters::new(new_browsing_context, None))
            .await?;

        let final_windows = utils::browser::get_client_windows(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert_eq!(initial_windows.len(), 1);
        assert_eq!(initial_contexts.len(), 1);

        assert_eq!(initial_all_windows.len(), 2);
        assert!(initial_second_window.active);
        assert!(!initial_first_window.active);

        assert!(final_first_window.active);
        assert!(!final_second_window.active);

        assert!(final_windows[0].active);
        assert_eq!(final_windows[0].client_window, *initial_window_id);

        Ok(())
    }
}

// --------------------------------------------------

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/get_user_contexts
mod get_user_contexts {
    use super::*;

    #[tokio::test]
    async fn test_default() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let user_context_ids = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert!(!user_context_ids.is_empty());
        assert!(user_context_ids.contains(&DEFAULT_USER_CONTEXT.to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_remove_contexts() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let user_context_1 = utils::browser::create_user_context(&mut bidi_session).await?;
        let user_context_2 = utils::browser::create_user_context(&mut bidi_session).await?;

        let user_context_ids_1 = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::browser::remove_user_context(&mut bidi_session, user_context_1.clone()).await?;

        let user_context_ids_2 = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::browser::remove_user_context(&mut bidi_session, user_context_2.clone()).await?;

        let user_context_ids_3 = utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert!(user_context_ids_1.len() >= 3);
        assert!(user_context_ids_1.contains(&user_context_1));
        assert!(user_context_ids_1.contains(&user_context_2));
        assert!(user_context_ids_1.contains(&DEFAULT_USER_CONTEXT.to_string()));

        assert!(!user_context_ids_2.contains(&user_context_1));
        assert!(user_context_ids_2.contains(&user_context_2));
        assert!(user_context_ids_2.contains(&DEFAULT_USER_CONTEXT.to_string()));

        assert!(!user_context_ids_3.contains(&user_context_2));
        assert!(user_context_ids_3.contains(&DEFAULT_USER_CONTEXT.to_string()));

        Ok(())
    }
}

// // --------------------------------------------------

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/remove_user_context
mod remove_user_context {
    use super::*;

    const BROWSING_CTX_DESTROYED_EVENT: &str = "browsingContext.contextDestroyed";
    // const USER_PROMPT_OPENED_EVENT: &str = "browsingContext.userPromptOpened";

    #[tokio::test]
    async fn test_remove_context() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let user_context = utils::browser::create_user_context(&mut bidi_session).await?;
        let initial_user_context_ids =
            utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::browser::remove_user_context(&mut bidi_session, user_context.clone()).await?;

        let final_user_context_ids =
            utils::browser::get_user_context_ids(&mut bidi_session).await?;

        utils::session::close(&mut bidi_session).await?;

        assert!(initial_user_context_ids.contains(&user_context));

        assert!(!final_user_context_ids.contains(&user_context));
        assert!(final_user_context_ids.contains(&DEFAULT_USER_CONTEXT.to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_context_closes_contexts() -> Result<()> {
        let mut bidi_session = utils::session::init().await?;

        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));

        {
            let events = events.clone();

            bidi_session
                .register_event_handler(
                    EventType::BrowsingContextContextDestroyed,
                    move |event: serde_json::Value| {
                        let events = events.clone();
                        async move {
                            debug!(
                                "Received browsingContext.contextDestroyed event: {:?}",
                                event
                            );
                            events.lock().await.push(event);
                        }
                    },
                )
                .await;
        }

        bidi_session
            .session_subscribe(SubscriptionRequest::new(
                vec![String::from(BROWSING_CTX_DESTROYED_EVENT)],
                None,
                None,
            ))
            .await?;

        let user_context_1 = utils::browser::create_user_context(&mut bidi_session).await?;
        let user_context_2 = utils::browser::create_user_context(&mut bidi_session).await?;

        let context_1 = utils::browsing_context::new_tab_in_user_context(
            &mut bidi_session,
            user_context_1.clone(),
        )
        .await?;

        let context_2 = utils::browsing_context::new_tab_in_user_context(
            &mut bidi_session,
            user_context_1.clone(),
        )
        .await?;

        let context_3 = utils::browsing_context::new_tab_in_user_context(
            &mut bidi_session,
            user_context_2.clone(),
        )
        .await?;

        let context_4 = utils::browsing_context::new_tab_in_user_context(
            &mut bidi_session,
            user_context_2.clone(),
        )
        .await?;

        bidi_session
            .browser_remove_user_context(RemoveUserContextParameters::new(user_context_1))
            .await?;

        let initial_events_len = events.lock().await.len();

        let initial_destroyed_contexts = events
            .lock()
            .await
            .iter()
            .filter_map(|event| event["parasm"]["context"].as_str().map(String::from))
            .collect::<Vec<_>>();

        bidi_session
            .browser_remove_user_context(RemoveUserContextParameters::new(user_context_2))
            .await?;

        utils::session::close(&mut bidi_session).await?;

        let final_events_len = events.lock().await.len();

        let final_destroyed_contexts = events
            .lock()
            .await
            .iter()
            .filter_map(|event| event["parasm"]["context"].as_str().map(String::from))
            .collect::<Vec<_>>();

        assert!(initial_events_len == 2);
        assert!(initial_destroyed_contexts.contains(&context_1));
        assert!(initial_destroyed_contexts.contains(&context_2));

        assert!(final_events_len == 4);
        assert!(final_destroyed_contexts.contains(&context_3));
        assert!(final_destroyed_contexts.contains(&context_4));

        Ok(())
    }
}

//     #[tokio::test]
//     async fn test_remove_context_skips_beforeunload_prompt() {
//         let mut bidi_session = utils::init_session().await?;

//         bidi_session
//             .session_subscribe(SubscriptionRequest::new(
//                 vec![String::from(USER_PROMPT_OPENED_EVENT)],
//                 None,
//                 None,
//             ))
//             .await
//             ?;

//         let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));

//         {
//             let events = events.clone();

//             bidi_session
//                 .register_event_handler(
//                     EventType::BrowsingContextUserPromptOpened,
//                     move |event: serde_json::Value| {
//                         let events = events.clone();
//                         async move {
//                             debug!(
//                                 "Received browsingContext.userPromptOpened event: {:?}",
//                                 event
//                             );
//                             events.lock().await.push(event);
//                         }
//                     },
//                 )
//                 .await;
//         }

//         let addr = format!("{}:0", utils::HOST);
//         // utils::serve_static_html("./static/beforeunload.html").await
//         // let server = HttpServer::new(|| App::new().route("/", web::get().to(before_unload)))
//         let server = HttpServer::new(|| {
//             App::new().route(
//                 "/",
//                 web::get()
//                     .to(async || utils::serve_static_html("./static/beforeunload.html").await),
//             )
//         })
//         .bind(addr)
//         ?;

//         let addr = server.addrs()[0];
//         let url = format!("http://{}", addr);
//         let server_handle = tokio::spawn(server.run());
//         utils::sleep_for_millis(100).await;

//         let user_context = utils::create_user_context(&mut bidi_session).await?;
//         let context = utils::new_tab_in_user_context(&mut bidi_session, user_context.clone())
//             .await
//             ?;

//         utils::navigate(&mut bidi_session, context, url)
//             .await
//             ?;

//         utils::remove_user_context(&mut bidi_session, user_context)
//             .await
//             ?;

//         utils::close_session(&mut bidi_session).await?;
//         server_handle.abort();

//         // debug!("Events vec len: {}", events.lock().await.len());
//         assert!(events.lock().await.len() > 0);
//     }
// }
