use anyhow::Result;

use webdriverbidi::model::browsing_context::ActivateParameters;
use webdriverbidi::model::script::{ContextTarget, EvaluateParameters, Target};

mod utils;

// // https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/activate
// mod activate {

//     use super::*;

//     #[tokio::test]
//     async fn test_switch_between_contexts() -> Result<()> {
//         let mut bidi_session = utils::session::init().await?;

//         let top_context = utils::browsing_context::get_nth_context(&mut bidi_session, 0).await?;
//         let new_context = utils::browsing_context::new_tab(&mut bidi_session).await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(top_context.clone()))
//             .await?;

//         let top_context_initial_status =
//             utils::assert_document_status(&mut bidi_session, &top_context).await?;
//         let new_context_initial_status =
//             utils::assert_document_status(&mut bidi_session, &new_context).await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(new_context.clone()))
//             .await?;

//         let top_context_final_status =
//             utils::assert_document_status(&mut bidi_session, &top_context).await?;
//         let new_context_final_status =
//             utils::assert_document_status(&mut bidi_session, &new_context).await?;

//         utils::session::close(&mut bidi_session).await?;

//         assert!(top_context_initial_status);
//         assert!(!new_context_initial_status);

//         assert!(!top_context_final_status);
//         assert!(new_context_final_status);

//         return Ok(());
//     }

//     const TEST_KEEPS_ELEMENT_FOCUSED: &str = "";

//     #[tokio::test]
//     async fn test_keeps_element_focused() -> Result<()> {
//         let mut bidi_session = utils::session::init().await?;

//         let top_context = utils::browsing_context::get_nth_context(&mut bidi_session, 0).await?;

//         let (url, server_handle) =
//             utils::axum_utils::serve_static(TEST_KEEPS_ELEMENT_FOCUSED).await?;

//         let new_tab = utils::browsing_context::new_tab(&mut bidi_session).await?;

//         utils::browsing_context::navigate(&mut bidi_session, new_tab.clone(), url.clone()).await?;

//         let params = EvaluateParameters::new(
//             r#"document.querySelector("input").focus()"#.to_string(),
//             Target::ContextTarget(ContextTarget::new(new_tab.clone(), None)),
//             false,
//             None,
//             None,
//             None,
//         );

//         bidi_session.script_evaluate(params).await?;

//         let is_focused_1 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(top_context))
//             .await?;

//         let is_focused_2 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(new_tab.clone()))
//             .await?;

//         let is_focused_3 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         utils::session::close(&mut bidi_session).await?;
//         server_handle.abort();

//         assert!(is_focused_1);
//         assert!(is_focused_2);
//         assert!(is_focused_3);

//         Ok(())
//     }

//     const TEST_MULTIPLE_ACTIVATION_HTML: &str = "test_multiple_activation.html";

//     #[tokio::test]
//     async fn test_multiple_activation() -> Result<()> {
//         let (url, server_handle) =
//             utils::axum_utils::serve_static(TEST_MULTIPLE_ACTIVATION_HTML).await?;

//         let mut bidi_session = utils::session::init().await?;

//         let new_tab = utils::browsing_context::new_tab(&mut bidi_session).await?;
//         utils::browsing_context::navigate(&mut bidi_session, new_tab.clone(), url.clone()).await?;

//         let initial_status = utils::assert_document_status(&mut bidi_session, &new_tab).await?;
//         let is_focused_1 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(new_tab.clone()))
//             .await?;
//         let middle_status = utils::assert_document_status(&mut bidi_session, &new_tab).await?;
//         let is_focused_2 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         bidi_session
//             .browsing_context_activate(ActivateParameters::new(new_tab.clone()))
//             .await?;
//         let final_status = utils::assert_document_status(&mut bidi_session, &new_tab).await?;
//         let is_focused_3 =
//             utils::is_element_focused(&mut bidi_session, new_tab.as_str(), "input").await?;

//         utils::session::close(&mut bidi_session).await?;
//         server_handle.abort();

//         assert!(initial_status);
//         assert!(is_focused_1);
//         assert!(middle_status);
//         assert!(is_focused_2);
//         assert!(final_status);
//         assert!(is_focused_3);

//         return Ok(());
//     }
// }

// // // async def get_viewport_dimensions(bidi_session, context: str,
// // //       with_scrollbar: bool = True, quirk_mode: bool = False):
// // //     if with_scrollbar:
// // //         expression = """
// // //             ({
// // //                 height: window.innerHeight,
// // //                 width: window.innerWidth,
// // //             });
// // //         """
// // //     else:
// // //         # The way the viewport height without the scrollbar can be calculated
// // //         # is different in quirks mode. In quirks mode, the viewport height is
// //         # the height of the body element, while in standard mode it is the
// //         # height of the document element.
// //         element_expression = \
// //             "document.body" if quirk_mode else "document.documentElement"
// //         expression = f"""
// //             ({{
// //                 height: {element_expression}.clientHeight,
// //                 width: {element_expression}.clientWidth,
// //             }});
// //         """
// //     result = await bidi_session.script.evaluate(
// //         expression=expression,
// //         target=ContextTarget(context["context"]),
// //         await_promise=False,
// //     )

// //     return remote_mapping_to_dict(result["value"])

// // async def get_physical_viewport_dimensions(bidi_session, context):
// //     """Get the physical dimensions of the context's viewport.

// //     :param bidi_session: BiDiSession
// //     :param context: Browsing context ID
// //     :returns: Tuple of (int, int) containing viewport width, viewport height.
// //     """
// //     viewport = await get_viewport_dimensions(bidi_session, context)
// //     dpr = await get_device_pixel_ratio(bidi_session, context)
// //     return (floor(viewport["width"] * dpr), floor(viewport["height"] * dpr))

// // https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/capture_screenshot
// mod capture_screenshot {
//     use super::*;

//     // async def test_capture(bidi_session, top_context, inline, compare_png_bidi,
//     //                        activate):
//     //     expected_size = await get_physical_viewport_dimensions(bidi_session, top_context)

//     //     await bidi_session.browsing_context.navigate(
//     //         context=top_context["context"], url="about:blank", wait="complete"
//     //     )
//     //     if activate:
//     //         await bidi_session.browsing_context.activate(
//     //             context=top_context["context"])
//     //     reference_data = await bidi_session.browsing_context.capture_screenshot(
//     //         context=top_context["context"])
//     //     assert png_dimensions(reference_data) == expected_size

//     //     await bidi_session.browsing_context.navigate(
//     //         context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
//     //     )
//     //     if activate:
//     //         await bidi_session.browsing_context.activate(
//     //             context=top_context["context"])
//     //     data = await bidi_session.browsing_context.capture_screenshot(
//     //         context=top_context["context"])

//     //     comparison = await compare_png_bidi(data, reference_data)
//     //     assert not comparison.equal()

//     //     # Take a second screenshot that should be identical to validate that
//     //     # we don't just always return false here
//     //     await bidi_session.browsing_context.navigate(
//     //         context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
//     //     )
//     //     if activate:
//     //         await bidi_session.browsing_context.activate(
//     //             context=top_context["context"])
//     //     new_data = await bidi_session.browsing_context.capture_screenshot(
//     //         context=top_context["context"])

//     //     comparison = await compare_png_bidi(new_data, data)
//     //     assert comparison.equal()

//     // @pytest.mark.parametrize("delta_width", [-10, +20], ids=["width smaller", "width larger"])
//     // @pytest.mark.parametrize("delta_height", [-30, +40], ids=["height smaller", "height larger"])
//     // @pytest.mark.asyncio
//     // async def test_capture_with_viewport(bidi_session, new_tab, delta_width, delta_height):
//     //     original_viewport = await get_viewport_dimensions(bidi_session, new_tab)

//     //     dpr = await get_device_pixel_ratio(bidi_session, new_tab)

//     //     test_viewport = {
//     //         "width": original_viewport["width"] + delta_width,
//     //         "height": original_viewport["height"] + delta_height
//     //     }
//     //     await bidi_session.browsing_context.set_viewport(
//     //         context=new_tab["context"],
//     //         viewport=test_viewport)

//     //     expected_size = {
//     //         "width": floor(test_viewport["width"] * dpr),
//     //         "height": floor(test_viewport["height"] * dpr)
//     //     }

//     //     await bidi_session.browsing_context.navigate(
//     //         context=new_tab["context"], url="about:blank", wait="complete"
//     //     )

//     //     result = await bidi_session.browsing_context.capture_screenshot(
//     //         context=new_tab["context"])
//     //     assert png_dimensions(result) == (expected_size["width"], expected_size["height"])

//     // @pytest.mark.parametrize("dpr", [0.5, 2])
//     // @pytest.mark.asyncio
//     // async def test_capture_with_different_dpr(bidi_session, new_tab, inline, dpr):
//     //     page = inline("<div style='background-color: black; width: 100px; height: 100px;'></div>")
//     //     await bidi_session.browsing_context.navigate(
//     //         context=new_tab["context"], url=page, wait="complete"
//     //     )

//     //     original_viewport = await get_viewport_dimensions(bidi_session, new_tab)

//     //     await bidi_session.browsing_context.set_viewport(
//     //         context=new_tab["context"],
//     //         device_pixel_ratio=dpr)

//     //     expected_width = original_viewport["width"] * dpr
//     //     expected_height = original_viewport["height"] * dpr

//     //     data = await bidi_session.browsing_context.capture_screenshot(context=new_tab["context"])
//     //     (actual_width, actual_height) = png_dimensions(data)
//     //     # The rounding is implementation-specific and can be either floor, ceil or round depending on the browser
//     //     # implementation. Tolerate any value between floor and ceil.
//     //     assert floor(expected_width) <= actual_width <= ceil(expected_width)
//     //     assert floor(expected_height) <= actual_height <= ceil(expected_height)
// }
