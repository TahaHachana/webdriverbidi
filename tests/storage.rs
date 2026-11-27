//! Live tests for the `storage` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::browser::CreateUserContextParameters;
use webdriverbidi::model::browsing_context::{CreateParameters, CreateType};
use webdriverbidi::model::common::Extensible;
use webdriverbidi::model::network::{BytesValue, Cookie, SameSite, StringValue};
use webdriverbidi::model::storage::*;
use webdriverbidi::session::WebDriverBiDiSession;

use support::{eval_string, inline, navigate, server_host, server_url, top_context, with_session};

fn text(value: &str) -> BytesValue {
    BytesValue::StringValue(StringValue {
        value_type: "string".into(),
        value: value.into(),
    })
}

fn named(name: &str) -> CookieFilter {
    CookieFilter::new(
        Some(name.into()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Extensible::new(),
    )
}

fn context_partition(context: &str) -> PartitionDescriptor {
    PartitionDescriptor::BrowsingContextPartitionDescriptor(
        BrowsingContextPartitionDescriptor::new(context.into()),
    )
}

async fn cookies_named(
    session: &mut WebDriverBiDiSession,
    name: &str,
    partition: Option<PartitionDescriptor>,
) -> Result<Vec<Cookie>> {
    Ok(session
        .storage_get_cookies(GetCookiesParameters::new(Some(named(name)), partition))
        .await?
        .cookies)
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/storage/set_cookie
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/storage/get_cookies
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/storage/delete_cookies
#[tokio::test]
async fn set_get_and_delete_cookies() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>cookies</p>")).await?;

        let result = session
            .storage_set_cookie(SetCookieParameters::new(
                PartialCookie::new(
                    "flavour".into(),
                    text("chocolate"),
                    server_host(),
                    Some("/".into()),
                    Some(false),
                    Some(false),
                    Some(SameSite::Lax),
                    None,
                    Extensible::new(),
                ),
                Some(context_partition(&context)),
            ))
            .await?;
        ensure!(
            result.partition_key.user_context.is_some(),
            "partition key has no user context"
        );

        let page_cookies = eval_string(&mut session, &context, "document.cookie").await?;
        ensure!(
            page_cookies.contains("flavour=chocolate"),
            "page sees {page_cookies:?}"
        );

        let cookies =
            cookies_named(&mut session, "flavour", Some(context_partition(&context))).await?;
        ensure!(cookies.len() == 1, "found {} cookies", cookies.len());
        let cookie = &cookies[0];
        ensure!(
            matches!(&cookie.value, BytesValue::StringValue(v) if v.value == "chocolate"),
            "cookie value is {:?}",
            cookie.value
        );
        ensure!(
            cookie.domain == server_host(),
            "cookie domain is {}",
            cookie.domain
        );
        ensure!(
            matches!(cookie.same_site, SameSite::Lax),
            "sameSite is {:?}",
            cookie.same_site
        );

        session
            .storage_delete_cookies(DeleteCookiesParameters::new(
                Some(named("flavour")),
                Some(context_partition(&context)),
            ))
            .await?;
        ensure!(
            cookies_named(&mut session, "flavour", Some(context_partition(&context)))
                .await?
                .is_empty(),
            "cookie survived deleteCookies"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn get_cookies_sees_server_set_cookies() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &server_url("/cookie")).await?;
        let cookies =
            cookies_named(&mut session, "served", Some(context_partition(&context))).await?;
        ensure!(
            cookies.len() == 1,
            "found {} `served` cookies",
            cookies.len()
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn storage_key_partition_targets_a_user_context() -> Result<()> {
    with_session(|mut session| async move {
        let user_context = session
            .browser_create_user_context(CreateUserContextParameters::new(None, None, None))
            .await?
            .user_context;
        let partition = || {
            PartitionDescriptor::StorageKeyPartitionDescriptor(StorageKeyPartitionDescriptor::new(
                Some(user_context.clone()),
                None,
            ))
        };
        session
            .storage_set_cookie(SetCookieParameters::new(
                PartialCookie::new(
                    "scoped".into(),
                    text("yes"),
                    server_host(),
                    Some("/".into()),
                    None,
                    None,
                    None,
                    None,
                    Extensible::new(),
                ),
                Some(partition()),
            ))
            .await?;

        let tab = session
            .browsing_context_create(CreateParameters::new(
                CreateType::Tab,
                None,
                None,
                Some(user_context.clone()),
            ))
            .await?
            .context;
        navigate(&mut session, &tab, &inline("<p>scoped</p>")).await?;
        let seen = eval_string(&mut session, &tab, "document.cookie").await?;
        ensure!(
            seen.contains("scoped=yes"),
            "the user context's tab sees {seen:?}"
        );

        let default_context = top_context(&mut session).await?;
        navigate(&mut session, &default_context, &inline("<p>default</p>")).await?;
        let leaked = eval_string(&mut session, &default_context, "document.cookie").await?;
        ensure!(
            !leaked.contains("scoped"),
            "cookie leaked to the default user context"
        );

        ensure!(
            cookies_named(&mut session, "scoped", Some(partition()))
                .await?
                .len()
                == 1,
            "getCookies by storage key missed the cookie"
        );
        Ok(())
    })
    .await
}
