//! I define `www-authenticate` typed header and related types

use headers::{Header, HeaderName, HeaderValue};

mod challenge;
mod token68;

pub use challenge::*;
pub use token68::*;

/// `WWW-Authenticate` header is defined in [`rfc9110`](https://www.rfc-editor.org/rfc/rfc9110.html#section-11.6.1)
///
/// The syntax of the WWW-Authenticate header:
///
/// ```txt
///     auth-scheme    = token
///     token68        = 1*( ALPHA / DIGIT /
///                                 "-" / "." / "_" / "~" / "+" / "/" ) *"="
///     auth-param     = token BWS "=" BWS ( token / quoted-string )
///     challenge   = auth-scheme [ 1*SP ( token68 / #auth-param ) ]
///     WWW-Authenticate = #challenge
///```
///  The field value consists of a single URI-reference.
/// When it has the form of a relative reference,
/// the final value is computed by resolving it against the
///  target URI.
///
/// NOTE: this implementation doesn't support multiple challenges in same field value. As noted in rfc:
/// >
#[derive(Debug, Clone)]
pub struct WWWAuthenticate {
    /// List of challenges.
    pub challenges: Vec<Challenge>,
}

/// Static for `www-authenticate` header-name.
pub static WWW_AUTHENTICATE: HeaderName = HeaderName::from_static("www-authenticate");

impl Header for WWWAuthenticate {
    #[inline]
    fn name() -> &'static HeaderName {
        &WWW_AUTHENTICATE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let challenges = values
            .map(|v| {
                v.to_str()
                    .map_err(|_| headers::Error::invalid())
                    .and_then(|v| v.parse().map_err(|_| headers::Error::invalid()))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { challenges })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(
            self.challenges.iter().map(|c| unsafe {
                HeaderValue::from_maybe_shared_unchecked(c.encode().into_bytes())
            }),
        )
    }
}

// TODO tests.
