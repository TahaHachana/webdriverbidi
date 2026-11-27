//! Live tests for the `emulation` module. Needs a WebDriver BiDi server; see `tests/support`.
//!
//! Each test applies an override to one context, checks its effect from page
//! script where the effect is observable, and resets it.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::browser::CreateUserContextParameters;
use webdriverbidi::model::browsing_context::{CreateParameters, CreateType};
use webdriverbidi::model::emulation::*;
use webdriverbidi::session::WebDriverBiDiSession;

use support::{
    error_code, eval_string, inline, is_firefox, navigate, supported, top_context, with_session,
};

/// Load a fresh page, so overrides that apply at navigation take effect.
async fn fresh_page(session: &mut WebDriverBiDiSession, context: &str) -> Result<()> {
    navigate(session, context, &inline("<p>emulation</p>")).await
}

async fn expect(
    session: &mut WebDriverBiDiSession,
    context: &str,
    expression: &str,
    expected: &str,
) -> Result<()> {
    let got = eval_string(session, context, expression).await?;
    ensure!(
        got == expected,
        "`{expression}` is {got:?}, expected {expected:?}"
    );
    Ok(())
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_user_agent_override
#[tokio::test]
async fn user_agent_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let original = eval_string(&mut session, &context, "navigator.userAgent").await?;
        let set = session
            .set_user_agent_override(SetUserAgentOverrideParameters::new(
                Some("TestAgent/1.0".into()),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setUserAgentOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &context).await?;
        expect(
            &mut session,
            &context,
            "navigator.userAgent",
            "TestAgent/1.0",
        )
        .await?;

        session
            .set_user_agent_override(SetUserAgentOverrideParameters::new(
                None,
                Some(vec![context.clone()]),
                None,
            ))
            .await?;
        fresh_page(&mut session, &context).await?;
        expect(&mut session, &context, "navigator.userAgent", &original).await
    })
    .await
}

#[tokio::test]
async fn override_for_a_user_context() -> Result<()> {
    with_session(|mut session| async move {
        let user_context = session
            .browser_create_user_context(CreateUserContextParameters::new(None, None, None))
            .await?
            .user_context;
        let tab = session
            .browsing_context_create(CreateParameters::new(
                CreateType::Tab,
                None,
                None,
                Some(user_context.clone()),
            ))
            .await?
            .context;
        let set = session
            .set_user_agent_override(SetUserAgentOverrideParameters::new(
                Some("UserContextAgent/1.0".into()),
                None,
                Some(vec![user_context]),
            ))
            .await;
        if supported("emulation.setUserAgentOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &tab).await?;
        expect(
            &mut session,
            &tab,
            "navigator.userAgent",
            "UserContextAgent/1.0",
        )
        .await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_locale_override
#[tokio::test]
async fn locale_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_locale_override(SetLocaleOverrideParameters {
                locale: Some("de-DE".into()),
                contexts: Some(vec![context.clone()]),
                user_contexts: None,
            })
            .await;
        if supported("emulation.setLocaleOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &context).await?;
        expect(
            &mut session,
            &context,
            "Intl.DateTimeFormat().resolvedOptions().locale",
            "de-DE",
        )
        .await?;
        session
            .set_locale_override(SetLocaleOverrideParameters {
                locale: None,
                contexts: Some(vec![context]),
                user_contexts: None,
            })
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_timezone_override
#[tokio::test]
async fn timezone_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_timezone_override(SetTimezoneOverrideParameters {
                timezone: Some("Asia/Tokyo".into()),
                contexts: Some(vec![context.clone()]),
                user_contexts: None,
            })
            .await;
        if supported("emulation.setTimezoneOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &context).await?;
        expect(
            &mut session,
            &context,
            "Intl.DateTimeFormat().resolvedOptions().timeZone",
            "Asia/Tokyo",
        )
        .await?;
        session
            .set_timezone_override(SetTimezoneOverrideParameters {
                timezone: None,
                contexts: Some(vec![context]),
                user_contexts: None,
            })
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_geolocation_override
#[tokio::test]
async fn geolocation_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let mut coordinates = GeolocationCoordinates::new(48.8584, 2.2945);
        coordinates.accuracy = 5.0;
        let set = session
            .set_geolocation_override(SetGeolocationOverrideParameters::with_coordinates(
                Some(coordinates),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setGeolocationOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &context).await?;
        // Reading the position also needs the geolocation permission, which this
        // crate can't grant (no `permissions` module), so accept a permission
        // error but nothing else.
        let position = eval_string(
            &mut session,
            &context,
            "Promise.race([
                new Promise(resolve => navigator.geolocation.getCurrentPosition(
                    p => resolve(`${p.coords.latitude},${p.coords.longitude}`),
                    e => resolve(`error:${e.code}`))),
                // Firefox leaves the permission request pending.
                new Promise(resolve => setTimeout(() => resolve('pending'), 3000))])",
        )
        .await?;
        ensure!(
            position == "48.8584,2.2945" || position == "error:1" || position == "pending",
            "position is {position}"
        );

        let errored = session
            .set_geolocation_override(SetGeolocationOverrideParameters::with_error(
                GeolocationPositionError::position_unavailable(),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        match errored {
            // Firefox 161 rejects the spec's `error` form ("Expected `coordinates`
            // to be an object"); the JSON is checked against the spec elsewhere.
            Err(e) if is_firefox() && error_code(&e) == Some("invalid argument") => {
                eprintln!("NOTE: known Firefox gap, geolocation `error` override rejected: {e}")
            }
            other => {
                other?;
            }
        }
        session
            .set_geolocation_override(SetGeolocationOverrideParameters::with_coordinates(
                None,
                Some(vec![context]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_screen_orientation_override
#[tokio::test]
async fn screen_orientation_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_screen_orientation_override(SetScreenOrientationOverrideParameters {
                screen_orientation: Some(ScreenOrientation {
                    natural: ScreenOrientationNatural::Portrait,
                    orientation_type: ScreenOrientationType::LandscapePrimary,
                }),
                contexts: Some(vec![context.clone()]),
                user_contexts: None,
            })
            .await;
        if supported("emulation.setScreenOrientationOverride", set)?.is_none() {
            return Ok(());
        }
        expect(
            &mut session,
            &context,
            "screen.orientation.type",
            "landscape-primary",
        )
        .await?;
        session
            .set_screen_orientation_override(SetScreenOrientationOverrideParameters {
                screen_orientation: None,
                contexts: Some(vec![context]),
                user_contexts: None,
            })
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_screen_settings_override
#[tokio::test]
async fn screen_settings_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_screen_settings_override(SetScreenSettingsOverrideParameters::new(
                Some(ScreenArea::new(1024, 768)),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setScreenSettingsOverride", set)?.is_none() {
            return Ok(());
        }
        expect(
            &mut session,
            &context,
            "`${screen.width}x${screen.height}`",
            "1024x768",
        )
        .await?;
        session
            .set_screen_settings_override(SetScreenSettingsOverrideParameters::new(
                None,
                Some(vec![context]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_media_features_override
#[tokio::test]
async fn media_features_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let features = MediaFeatures {
            prefers_color_scheme: Some(Some(PrefersColorScheme::Dark)),
            prefers_reduced_motion: Some(Some(PrefersReduced::Reduce)),
            ..Default::default()
        };
        let set = session
            .set_media_features_override(SetMediaFeaturesOverrideParameters::new(
                Some(features),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setMediaFeaturesOverride", set)?.is_none() {
            return Ok(());
        }
        expect(
            &mut session,
            &context,
            "String(matchMedia('(prefers-color-scheme: dark)').matches && \
             matchMedia('(prefers-reduced-motion: reduce)').matches)",
            "true",
        )
        .await?;
        session
            .set_media_features_override(SetMediaFeaturesOverrideParameters::new(
                None,
                Some(vec![context]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_forced_colors_mode_theme_override
#[tokio::test]
async fn forced_colors_mode_theme_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_forced_colors_mode_theme_override(SetForcedColorsModeThemeOverrideParameters::new(
                Some(ForcedColorsModeTheme::Dark),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setForcedColorsModeThemeOverride", set)?.is_none() {
            return Ok(());
        }
        expect(
            &mut session,
            &context,
            "String(matchMedia('(forced-colors: active)').matches)",
            "true",
        )
        .await?;
        session
            .set_forced_colors_mode_theme_override(SetForcedColorsModeThemeOverrideParameters::new(
                None,
                Some(vec![context]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_network_conditions
#[tokio::test]
async fn network_conditions_offline() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        fresh_page(&mut session, &context).await?;
        let set = session
            .set_network_conditions(SetNetworkConditionsParameters::new(
                Some(NetworkConditions::Offline),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setNetworkConditions", set)?.is_none() {
            return Ok(());
        }
        expect(&mut session, &context, "String(navigator.onLine)", "false").await?;
        expect(
            &mut session,
            &context,
            "fetch(location.href).then(() => 'fetched', () => 'failed')",
            "failed",
        )
        .await?;
        session
            .set_network_conditions(SetNetworkConditionsParameters::new(
                None,
                Some(vec![context.clone()]),
                None,
            ))
            .await?;
        expect(&mut session, &context, "String(navigator.onLine)", "true").await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_scripting_enabled
#[tokio::test]
async fn scripting_disabled() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let page = inline("<script>document.title = 'scripted'</script>");
        let set = session
            .set_scripting_enabled(SetScriptingEnabledParameters::new(
                Some(ScriptingEnabled::Disabled),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setScriptingEnabled", set)?.is_none() {
            return Ok(());
        }
        navigate(&mut session, &context, &page).await?;
        let title = eval_string(&mut session, &context, "document.title").await?;
        ensure!(
            title != "scripted",
            "page script ran with scripting disabled"
        );

        session
            .set_scripting_enabled(SetScriptingEnabledParameters::new(
                None,
                Some(vec![context.clone()]),
                None,
            ))
            .await?;
        navigate(&mut session, &context, &page).await?;
        expect(&mut session, &context, "document.title", "scripted").await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation/set_touch_override
#[tokio::test]
async fn touch_override() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let set = session
            .set_touch_override(SetTouchOverrideParameters::new(
                Some(5),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("emulation.setTouchOverride", set)?.is_none() {
            return Ok(());
        }
        fresh_page(&mut session, &context).await?;
        expect(
            &mut session,
            &context,
            "String(navigator.maxTouchPoints)",
            "5",
        )
        .await?;
        session
            .set_touch_override(SetTouchOverrideParameters::new(
                None,
                Some(vec![context]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// These overrides have no effect that headless Chrome exposes reliably to page
// script, so only check that each is accepted and can be reset.
#[tokio::test]
async fn scrollbar_text_layout_and_viewport_meta_overrides() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let contexts = || Some(vec![context.clone()]);

        let set = session
            .set_scrollbar_type_override(SetScrollbarTypeOverrideParameters::new(
                Some(ScrollbarType::Overlay),
                contexts(),
                None,
            ))
            .await;
        if supported("emulation.setScrollbarTypeOverride", set)?.is_some() {
            session
                .set_scrollbar_type_override(SetScrollbarTypeOverrideParameters::new(
                    None,
                    contexts(),
                    None,
                ))
                .await?;
        }

        let set = session
            .set_text_layout_mode_override(SetTextLayoutModeOverrideParameters::new(
                Some(TextLayoutMode::Mobile),
                contexts(),
                None,
            ))
            .await;
        if supported("emulation.setTextLayoutModeOverride", set)?.is_some() {
            session
                .set_text_layout_mode_override(SetTextLayoutModeOverrideParameters::new(
                    None,
                    contexts(),
                    None,
                ))
                .await?;
        }

        let set = session
            .set_viewport_meta_override(SetViewportMetaOverrideParameters::new(
                Some(ViewportMeta::Enabled),
                contexts(),
                None,
            ))
            .await;
        if supported("emulation.setViewportMetaOverride", set)?.is_some() {
            session
                .set_viewport_meta_override(SetViewportMetaOverrideParameters::new(
                    None,
                    contexts(),
                    None,
                ))
                .await?;
        }
        Ok(())
    })
    .await
}
