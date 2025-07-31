//! Cookie parsing and cookie jar management.
//!
//! See [`CookieJar`], [`SignedCookieJar`], and [`PrivateCookieJar`] for more details.

use axum::http::{
    header::{COOKIE, SET_COOKIE},
    request::Parts,
    HeaderMap,
};
use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use cookie::Cookie;
use std::convert::Infallible;

/// Extractor that grabs cookies from the request and manages the jar.
///
/// Note that methods like [`CookieJar::add`], [`CookieJar::remove`], etc updates the [`CookieJar`]
/// and returns it. This value _must_ be returned from the handler as part of the response for the
/// changes to be propagated.
///
/// # Example
///
/// ```rust
/// use axum::{
///     Router,
///     routing::{post, get},
///     response::{IntoResponse, Redirect},
///     http::StatusCode,
/// };
/// use axum_extra::{
///     TypedHeader,
///     headers::authorization::{Authorization, Bearer},
///     extract::cookie::{CookieJar, Cookie},
/// };
///
/// async fn create_session(
///     TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
///     jar: CookieJar,
/// ) -> Result<(CookieJar, Redirect), StatusCode> {
///     if let Some(session_id) = authorize_and_create_session(auth.token()).await {
///         Ok((
///             // the updated jar must be returned for the changes
///             // to be included in the response
///             jar.add(Cookie::new("session_id", session_id)),
///             Redirect::to("/me"),
///         ))
///     } else {
///         Err(StatusCode::UNAUTHORIZED)
///     }
/// }
///
/// async fn me(jar: CookieJar) -> Result<(), StatusCode> {
///     if let Some(session_id) = jar.get("session_id") {
///         // fetch and render user...
///         # Ok(())
///     } else {
///         Err(StatusCode::UNAUTHORIZED)
///     }
/// }
///
/// async fn authorize_and_create_session(token: &str) -> Option<String> {
///     // authorize the user and create a session...
///     # todo!()
/// }
///
/// let app = Router::new()
///     .route("/sessions", post(create_session))
///     .route("/me", get(me));
/// # let app: Router = app;
/// ```
#[must_use = "`CookieJar` should be returned as part of a `Response`, otherwise it does nothing."]
#[derive(Debug, Default, Clone)]
pub struct CookieJar {
    jar: cookie::CookieJar,
}

impl<S> FromRequestParts<S> for CookieJar
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_headers(&parts.headers))
    }
}

fn cookies_from_request(headers: &HeaderMap) -> impl Iterator<Item = Cookie<'static>> + '_ {
    headers
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(move |cookie| Cookie::parse(cookie.trim()).ok())
        .map(|cookie| {
            // Convert Cookie<'_> to Cookie<'static> by allocating owned name and value
            let name = cookie.name().to_owned();
            let value = cookie.value().to_owned();
            let mut owned_cookie = Cookie::new(name, value);
            // Copy over other attributes if needed
            if let Some(path) = cookie.path() {
                owned_cookie.set_path(path.to_owned());
            }
            if let Some(domain) = cookie.domain() {
                owned_cookie.set_domain(domain.to_owned());
            }
            if let Some(secure) = cookie.secure() {
                owned_cookie.set_secure(secure);
            }
            if let Some(http_only) = cookie.http_only() {
                owned_cookie.set_http_only(http_only);
            }
            if let Some(same_site) = cookie.same_site() {
                owned_cookie.set_same_site(same_site);
            }
            if let Some(max_age) = cookie.max_age() {
                owned_cookie.set_max_age(max_age);
            }
            if let Some(expires) = cookie.expires() {
                owned_cookie.set_expires(expires);
            }
            owned_cookie
        })
}

impl CookieJar {
    /// Create a new `CookieJar` from a map of request headers.
    ///
    /// The cookies in `headers` will be added to the jar.
    ///
    /// This is intended to be used in middleware and other places where it might be difficult to
    /// run extractors. Normally you should create `CookieJar`s through [`FromRequestParts`].
    ///
    /// [`FromRequestParts`]: axum::extract::FromRequestParts
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let mut jar = cookie::CookieJar::new();
        for cookie in cookies_from_request(headers) {
            jar.add_original(cookie);
        }
        Self { jar }
    }

    /// Create a new empty `CookieJar`.
    ///
    /// This is intended to be used in middleware and other places where it might be difficult to
    /// run extractors. Normally you should create `CookieJar`s through [`FromRequestParts`].
    ///
    /// If you need a jar that contains the headers from a request use `impl From<&HeaderMap> for
    /// CookieJar`.
    ///
    /// [`FromRequestParts`]: axum::extract::FromRequestParts
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cookie from the jar.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_extra::extract::cookie::CookieJar;
    /// use axum::response::IntoResponse;
    ///
    /// async fn handle(jar: CookieJar) {
    ///     let value: Option<String> = jar
    ///         .get("foo")
    ///         .map(|cookie| cookie.value().to_owned());
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.jar.get(name)
    }

    /// Remove a cookie from the jar.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_extra::extract::cookie::{CookieJar, Cookie};
    /// use axum::response::IntoResponse;
    ///
    /// async fn handle(jar: CookieJar) -> CookieJar {
    ///     jar.remove(Cookie::from("foo"))
    /// }
    /// ```
    pub fn remove<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.remove(cookie);
        self
    }

    /// Add a cookie to the jar.
    ///
    /// The value will automatically be percent-encoded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_extra::extract::cookie::{CookieJar, Cookie};
    /// use axum::response::IntoResponse;
    ///
    /// async fn handle(jar: CookieJar) -> CookieJar {
    ///     jar.add(Cookie::new("foo", "bar"))
    /// }
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add<C: Into<Cookie<'static>>>(mut self, cookie: C) -> Self {
        self.jar.add(cookie);
        self
    }

    /// Get an iterator over all cookies in the jar.
    pub fn iter(&self) -> impl Iterator<Item = &'_ Cookie<'static>> {
        self.jar.iter()
    }
}

impl IntoResponseParts for CookieJar {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        set_cookies(&self.jar, res.headers_mut());
        Ok(res)
    }
}

impl IntoResponse for CookieJar {
    fn into_response(self) -> Response {
        (self, ()).into_response()
    }
}

fn set_cookies(jar: &cookie::CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }

    // we don't need to call `jar.reset_delta()` because `into_response_parts` consumes the cookie
    // jar so it cannot be called multiple times.
}
