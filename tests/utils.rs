// use std::path::PathBuf;

// use actix_files::NamedFile;
use anyhow::Result;
use log::debug;
// use base64::prelude::*;
// use ctor::ctor;
// use simplelog::*;
// use std::fs::File;
// use std::io::Write;
// use tokio::time;

use axum::Router;
use axum::routing::get_service;
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
// --------------------------------------------------

use webdriverbidi::model::browser::ClientWindowInfo;
use webdriverbidi::model::browser::RemoveUserContextParameters;
use webdriverbidi::model::browsing_context::{
    CreateParameters,
    CreateType,
    // GetTreeParameters,
    // GetTreeParameters,
    NavigateParameters,
    ReadinessState,
    // TraverseHistoryParameters,
};
use webdriverbidi::model::common::EmptyParams;
use webdriverbidi::model::script::{
    CallFunctionParameters, ContextTarget, LocalValue, PrimitiveProtocolValue, StringValue, Target,
};
use webdriverbidi::model::script::{
    EvaluateResult,
    RemoteValue, // NumberOrSpecialNumber, NumberValue
};
use webdriverbidi::session::WebDriverBiDiSession;
use webdriverbidi::webdriver::capabilities::CapabilitiesRequest;

// --------------------------------------------------

const HOST: &str = "localhost";
const PORT: u16 = 4444;
// pub const TMP_ROUTE: &str = "/tmp.html";
const STATIC_ROUTE: &str = "/static";
const STATIC_DIR: &str = "static";
const DEFAULT_ADDR: &str = "127.0.0.1:0";

// --------------------------------------------------

pub mod session {
    use super::*;

    /// Initialize a new WebDriver BiDi session.
    pub async fn init() -> Result<WebDriverBiDiSession> {
        let capabilities = CapabilitiesRequest::default();
        let mut bidi_session = WebDriverBiDiSession::new(HOST.into(), PORT, capabilities);
        bidi_session.start().await?;

        Ok(bidi_session)
    }

    /// Close the supplied WebDriver BiDi session.
    pub async fn close(bidi_session: &mut WebDriverBiDiSession) -> Result<()> {
        bidi_session.close().await?;

        Ok(())
    }
}

pub mod browser {
    use super::*;

    /// Return the Ids of the current user contexts.
    pub async fn get_user_context_ids(
        bidi_session: &mut WebDriverBiDiSession,
    ) -> Result<Vec<String>> {
        let user_contexts = bidi_session
            .browser_get_user_contexts(EmptyParams::new())
            .await?
            .user_contexts;

        let user_contexts = user_contexts
            .into_iter()
            .map(|user_context_info| user_context_info.user_context)
            .collect::<Vec<_>>();

        Ok(user_contexts)
    }

    /// Create a user context.
    pub async fn create_user_context(bidi_session: &mut WebDriverBiDiSession) -> Result<String> {
        let user_context = bidi_session
            .browser_create_user_context(EmptyParams::new())
            .await?
            .user_context;

        Ok(user_context)
    }

    /// Retrieve the list of client windows.
    pub async fn get_client_windows(
        bidi_session: &mut WebDriverBiDiSession,
    ) -> Result<Vec<ClientWindowInfo>> {
        let client_windows = bidi_session
            .browser_get_client_windows(EmptyParams::new())
            .await?
            .client_windows;

        Ok(client_windows)
    }

    /// Remove a user context.
    pub async fn remove_user_context(
        bidi_session: &mut WebDriverBiDiSession,
        user_context: String,
    ) -> Result<()> {
        bidi_session
            .browser_remove_user_context(RemoveUserContextParameters::new(user_context.clone()))
            .await?;

        Ok(())
    }
}

pub mod browsing_context {
    use super::*;

    /// Open a new tab in the specified user context.
    pub async fn new_tab_in_user_context(
        session: &mut WebDriverBiDiSession,
        user_context: String,
    ) -> Result<String> {
        let create_params = CreateParameters::new(CreateType::Tab, None, None, Some(user_context));
        let context = session
            .browsing_context_create(create_params)
            .await?
            .context;

        Ok(context)
    }

    /// Navigate to the specified URL and wait for the document to completely load.
    pub async fn navigate(
        session: &mut WebDriverBiDiSession,
        context: String,
        url: String,
    ) -> Result<()> {
        let navigate_params = NavigateParameters::new(context, url, Some(ReadinessState::Complete));
        session.browsing_context_navigate(navigate_params).await?;

        Ok(())
    }

    /// Open a new window.
    pub async fn new_window(session: &mut WebDriverBiDiSession) -> Result<String> {
        let create_params = CreateParameters::new(CreateType::Window, None, None, None);
        let context = session
            .browsing_context_create(create_params)
            .await?
            .context;
        Ok(context)
    }
}

pub mod local_storage {
    use super::*;

    fn local_value(str: &str) -> LocalValue {
        LocalValue::PrimitiveProtocolValue(PrimitiveProtocolValue::StringValue(StringValue::new(
            str.to_string(),
        )))
    }

    fn target_context(context: &str) -> Target {
        Target::ContextTarget(ContextTarget::new(context.to_string(), None))
    }

    /// Return the value identified by the key from the context's localStorage.
    pub async fn get(
        bidi_session: &mut WebDriverBiDiSession,
        context: &str,
        key: &str,
    ) -> Result<Option<String>> {
        let function_declaration = "(key) => localStorage.getItem(key)".to_string();
        let key_local_value = local_value(key);
        let args = Some(vec![key_local_value]);
        let params = CallFunctionParameters::new(
            function_declaration,
            false,
            target_context(context),
            args,
            None,
            None,
            None,
            None,
        );
        let eval_result = bidi_session.script_call_function(params).await?;

        match eval_result {
            EvaluateResult::EvaluateResultSuccess(eval_rslt_success) => {
                let remote_value = eval_rslt_success.result;
                match remote_value {
                    RemoteValue::PrimitiveProtocolValue(
                        webdriverbidi::model::script::PrimitiveProtocolValue::StringValue(
                            string_value,
                        ),
                    ) => Ok(Some(string_value.value)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    /// Sets the value for the key in the context's localStorage.
    pub async fn set(
        bidi_session: &mut WebDriverBiDiSession,
        context: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let function_declaration = "(key, value) => localStorage.setItem(key, value)".to_string();
        let key_local_value = local_value(key);
        let value_local_value = local_value(value);
        let args = Some(vec![key_local_value, value_local_value]);
        let params = CallFunctionParameters::new(
            function_declaration,
            false,
            target_context(context),
            args,
            None,
            None,
            None,
            None,
        );
        bidi_session.script_call_function(params).await?;

        Ok(())
    }
}

pub mod axum_utils {
    use super::*;

    pub async fn serve_static(html_file: &str) -> Result<(String, JoinHandle<()>)> {
        let app = Router::new().nest_service(STATIC_ROUTE, get_service(ServeDir::new(STATIC_DIR)));
        let listener = tokio::net::TcpListener::bind(DEFAULT_ADDR).await?;
        let addr: SocketAddr = listener.local_addr()?;

        let server_handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                eprintln!("Server error: {}", e);
            }
        });

        let url = format!("http://{}{}/{}", addr, STATIC_ROUTE, html_file);
        debug!("Axum server running on {}", addr);

        Ok((url, server_handle))
    }
}

// // /// Sleep for a given number of seconds.
// // pub async fn sleep_for_secs(secs: u64) {
// //     time::sleep(time::Duration::from_secs(secs)).await
// // }

// /// Sleep for a given number of seconds.
// pub async fn sleep_for_millis(millis: u64) {
//     time::sleep(time::Duration::from_millis(millis)).await
// }

// // // --------------------------------------------------

// // /// Save a Base64-encoded screenshot to a file.
// // pub fn save_screenshot(base64_data: &str, file_path: &str) -> std::io::Result<()> {
// //     // Decode the Base64 string into bytes
// //     let decoded_data = BASE64_STANDARD
// //         .decode(base64_data)
// //         .expect("Failed to decode Base64 data");

// //     // Create a new file and write the decoded bytes
// //     let mut file = File::create(file_path)?;
// //     file.write_all(&decoded_data)?;

// //     println!("Screenshot saved to {}", file_path);
// //     Ok(())
// // }

// // // --------------------------------------------------

// /// Get the browsing context at the specified index.
// pub async fn get_nth_context(
//     session: &mut WebDriverBiDiSession,
//     index: usize,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let get_tree_params = GetTreeParameters::new(None, None);
//     let get_tree_rslt = session.browsing_context_get_tree(get_tree_params).await?;
//     Ok(get_tree_rslt.contexts[index].context.clone())
// }

// // /// Get the first browsing context from the browsing context tree.
// // pub async fn get_first_context(
// //     session: &mut WebDriverBiDiSession,
// // ) -> Result<String, Box<dyn std::error::Error>> {
// //     let get_tree_params = GetTreeParameters::new(None, None);
// //     let get_tree_rslt = session.browsing_context_get_tree(get_tree_params).await?;
// //     Ok(get_tree_rslt.contexts[0].context.clone())
// // }

// /// Open a new tab.
// pub async fn new_tab(session: &mut WebDriverBiDiSession) -> Result<String> {
//     let create_params = CreateParameters::new(CreateType::Tab, None, None, None);
//     let context = session
//         .browsing_context_create(create_params)
//         .await?
//         .context;
//     Ok(context)
// }

// // // --------------------------------------------------

// ///Initialize a simplelog TermLogger.
// #[ctor]
// fn init() {
//     TermLogger::init(
//         LevelFilter::Debug,
//         Config::default(),
//         TerminalMode::Mixed,
//         ColorChoice::Auto,
//     )
//     .unwrap();
// }

// // // --------------------------------------------------

// // pub async fn traverse_history(session: &mut WebDriverBiDiSession, context: String, delta: i64) {
// //     let traverse_history_params = TraverseHistoryParameters::new(context.clone(), delta);
// //     session
// //         .browsing_context_traverse_history(traverse_history_params)
// //         .await
// //         .expect("Failed to send command");
// // }

// pub mod inline {
//     use std::collections::HashMap;

//     use actix_web::{web, HttpResponse, Responder};
//     use serde::Deserialize;
//     use url::form_urlencoded;

//     /// A helper function that “inlines” a document by wrapping the given source
//     /// in a boilerplate template and then building a URL with the resulting document
//     /// embedded as a query parameter.
//     ///
//     /// The `build_url` parameter is a callback that takes a path and a query string
//     /// and returns a URL.
//     pub fn build_inline<F>(
//         build_url: F,
//         src: &str,
//         doctype: Option<&str>,
//         mime: Option<&str>,
//         charset: Option<&str>,
//         parameters: Option<HashMap<String, String>>,
//     ) -> String
//     where
//         F: Fn(&str, &str) -> String,
//     {
//         // Set default values.
//         let doctype = doctype.unwrap_or("html");
//         let mime = mime.unwrap_or(match doctype {
//             "html" | "html_quirks" => "text/html",
//             "xhtml" => "application/xhtml+xml",
//             "xml" => "text/xml",
//             "js" => "text/javascript",
//             _ => "text/html",
//         });
//         let charset = charset.unwrap_or("UTF-8");

//         // Choose the appropriate boilerplate.
//         let template = match doctype {
//             "html" => "<!doctype html>\n<meta charset={charset}>\n{src}",
//             "html_quirks" => "{src}",
//             "xhtml" => {
//                 r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN"
//         "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
//     <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
//       <head>
//         <title>XHTML might be the future</title>
//       </head>

//       <body>
//         {src}
//       </body>
//     </html>"#
//             }
//             "xml" => r#"<?xml version="1.0" encoding="{charset}"?>\n{src}"#,
//             "js" => "",
//             _ => "",
//         };

//         // Replace placeholders with provided values.
//         let doc = template.replace("{charset}", charset).replace("{src}", src);

//         // Build the query string with the required parameters.
//         let mut serializer = form_urlencoded::Serializer::new(String::new());
//         serializer.append_pair("doc", &doc);
//         serializer.append_pair("mime", mime);
//         serializer.append_pair("charset", charset);
//         if let Some(params) = parameters {
//             for (key, value) in params {
//                 serializer.append_pair(&key, &value);
//             }
//         }
//         let query = serializer.finish();

//         // Use the provided callback to build the full URL.
//         build_url(super::TMP_ROUTE, &query)
//     }

//     /// This struct is used to deserialize the query parameters passed to our handler.
//     #[derive(Deserialize)]
//     pub struct InlineQuery {
//         doc: Option<String>,
//         mime: Option<String>,
//         charset: Option<String>,
//     }

//     /// This function reads the query parameters (looking for “doc”, “mime”, and “charset”),
//     /// constructs a `Content-Type` header if appropriate, and returns the document
//     /// or an error message if “doc” is missing.
//     pub async fn inline_handler(query: web::Query<InlineQuery>) -> impl Responder {
//         // If the "doc" parameter is missing, return a 404 error.
//         if query.doc.is_none() {
//             return HttpResponse::NotFound()
//                 .content_type("text/plain")
//                 .body("Missing doc parameter in query");
//         }
//         let doc = query.doc.clone().unwrap();

//         // Build a content-type header (e.g. "text/html;charset=UTF-8") if provided.
//         let mut content_type_parts = Vec::new();
//         if let Some(mime) = &query.mime {
//             content_type_parts.push(mime.clone());
//         }
//         if let Some(charset) = &query.charset {
//             content_type_parts.push(format!("charset={}", charset));
//         }

//         let mut response = HttpResponse::Ok();
//         response.insert_header(("X-XSS-Protection", "0"));
//         if !content_type_parts.is_empty() {
//             let content_type_header = content_type_parts.join(";");
//             response.content_type(content_type_header);
//         }
//         response.body(doc)
//     }
// }

// pub async fn serve_static_html(path: &str) -> NamedFile {
//     let path: PathBuf = path.parse().unwrap();
//     NamedFile::open(path).unwrap()
// }

// pub async fn is_element_focused(
//     bidi_session: &mut WebDriverBiDiSession,
//     context: &str,
//     selector: &str,
// ) -> Result<bool> {
//     let function_declaration = "(selector) => {
//         return document.querySelector(selector) === document.activeElement;
//     }"
//     .to_string();
//     let selector_local_value = local_value(selector);
//     let args = Some(vec![selector_local_value]);
//     let params = CallFunctionParameters::new(
//         function_declaration,
//         false,
//         target_context(context),
//         args,
//         None,
//         None,
//         None,
//         None,
//     );
//     let rslt = bidi_session.script_call_function(params).await?;
//     debug!("is_element_focused result: {:?}", rslt);
//     Ok(true)
// }

// // async def is_element_focused(bidi_session, context: Mapping[str, Any], selector: str) -> bool:
// //     result = await bidi_session.script.call_function(
// //         function_declaration="""(selector) => {
// //         return document.querySelector(selector) === document.activeElement;
// //     }""",
// //         arguments=[
// //             {"type": "string", "value": selector},
// //         ],
// //         target=ContextTarget(context["context"]),
// //         await_promise=False)

// //     return result["value"]

// pub async fn assert_document_status(
//     bidi_session: &mut WebDriverBiDiSession,
//     context: &str,
// ) -> Result<bool> {
//     let visibility_state = get_visibility_state(bidi_session, context).await?;
//     let doc_focus = get_document_focus(bidi_session, context).await?;

//     Ok(visibility_state == "visible" && doc_focus)
//     // assert_eq!(visibility_state, "visible");
//     // assert_eq!(doc_focus, true);

//     // true
// }

// // async def assert_document_status(bidi_session, context, visible, focused):
// //     state = "visible" if visible else "hidden"

// //     assert await get_visibility_state(bidi_session, context) == state
// //     assert await get_document_focus(bidi_session, context) is focused
// //
// //

// pub async fn get_visibility_state(
//     bidi_session: &mut WebDriverBiDiSession,
//     context: &str,
// ) -> Result<String> {
//     let function_declaration = r#"() => {
//         return document.visibilityState;
//     }"#
//     .to_string();
//     let params = CallFunctionParameters::new(
//         function_declaration,
//         false,
//         target_context(context),
//         None,
//         None,
//         None,
//         None,
//         None,
//     );
//     let rslt = bidi_session.script_call_function(params).await?;
//     match rslt {
//         EvaluateResult::EvaluateResultSuccess(eval_rslt_success) => {
//             match eval_rslt_success.result {
//                 RemoteValue::PrimitiveProtocolValue(
//                     webdriverbidi::local::script::PrimitiveProtocolValue::StringValue(string_value),
//                 ) => {
//                     debug!("get_visibility_state result: {:?}", string_value);
//                     Ok(string_value.value)
//                 }
//                 remote_val => Err(anyhow::anyhow!(
//                 "Received EvaluateResultSuccess but not a string value, actual remote value: {:?}",
//                 remote_val
//             )),
//             }
//         }
//         eval_rslt => Err(anyhow::anyhow!(
//             "Received unexpected EvaluateResult: {:?}",
//             eval_rslt
//         )),
//     }
//     // debug!("get_visibility_state result: {:?}", rslt);
//     // // browsing_context::utils: get_visibility_state result: EvaluateResultSuccess(EvaluateResultSuccess { result_type: "success", result: PrimitiveProtocolValue(StringValue(StringValue { value_type: "string", value: "visible" })), realm: "030a41fd-d74b-4747-a4b1-5064432a2aeb" })

//     // Ok(String::from(""))
// }

// // async def get_visibility_state(bidi_session, context: Mapping[str, Any]) -> str:
// //     result = await bidi_session.script.call_function(
// //         function_declaration="""() => {
// //         return document.visibilityState;
// //     }""",
// //         target=ContextTarget(context["context"]),
// //         await_promise=False)
// //     return result["value"]

// pub async fn get_document_focus(
//     bidi_session: &mut WebDriverBiDiSession,
//     context: &str,
// ) -> Result<bool> {
//     let function_declaration = r#"() => {
//         return document.hasFocus();
//     }"#
//     .to_string();
//     let params = CallFunctionParameters::new(
//         function_declaration,
//         false,
//         target_context(context),
//         None,
//         None,
//         None,
//         None,
//         None,
//     );
//     let rslt = bidi_session.script_call_function(params).await?;
//     match rslt {
//         EvaluateResult::EvaluateResultSuccess(eval_rslt_success) => {
//             match eval_rslt_success.result {
//                 RemoteValue::PrimitiveProtocolValue(
//                     webdriverbidi::local::script::PrimitiveProtocolValue::BooleanValue(bool_value),
//                 ) => {
//                     debug!("get_document_focus result: {:?}", bool_value);
//                     Ok(bool_value.value)
//                 }
//                 remote_val => Err(anyhow::anyhow!(
//                 "Received EvaluateResultSuccess but not a boolean value, actual remote value: {:?}",
//                 remote_val
//             )),
//             }
//         }
//         eval_rslt => Err(anyhow::anyhow!(
//             "Received unexpected EvaluateResult: {:?}",
//             eval_rslt
//         )),
//     }

//     // debug!("get_document_focus result: {:?}", rslt);
//     // // browsing_context::utils: get_document_focus result: EvaluateResultSuccess(EvaluateResultSuccess { result_type: "success", result: PrimitiveProtocolValue(BooleanValue(BooleanValue { value_type: "boolean", value: true })), realm: "030a41fd-d74b-4747-a4b1-5064432a2aeb" })
//     // Ok(String::from(""))
// }

// async fn get_device_pixel_ratio(
//     bidi_session: &mut WebDriverBiDiSession,
//     context: &str,
// ) -> Result<f64> {
//     let function_declaration = r#"() => {
//         return window.devicePixelRatio;
//     }"#
//     .to_string();
//     let params = CallFunctionParameters::new(
//         function_declaration,
//         false,
//         target_context(context),
//         None,
//         None,
//         None,
//         None,
//         None,
//     );
//     let eval_result = bidi_session.script_call_function(params).await?;

//     match eval_result {
//         EvaluateResult::EvaluateResultSuccess(eval_rslt_success) => {
//             let remote_value = eval_rslt_success.result;
//             match remote_value {
//                 RemoteValue::PrimitiveProtocolValue(
//                     webdriverbidi::local::script::PrimitiveProtocolValue::NumberValue(NumberValue {
//                         value_type: _,
//                         value: NumberOrSpecialNumber::Number(number_value),
//                     }),
//                 ) => Ok(number_value),
//                 _ => Err(anyhow::anyhow!(
//                     "Received EvaluateResultSuccess but not a number value, actual remote value: {:?}",
//                     remote_value
//                 )),
//             }
//         }
//         _ => Err(anyhow::anyhow!(
//             "Received unexpected EvaluateResult: {:?}",
//             eval_result
//         )),
//     }
// }
