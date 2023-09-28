//! I define rust model for `access-param` abnf production.
//!

use std::str::FromStr;

use super::{access_mode::AccessMode, permission_group::PermissionGroup};
use crate::field::rules::parameter_value::DQUOTE;

/// A struct for representing `access-param` abnf production fro
/// [WAC specification](https://solid.github.io/web-access-control-spec/#wac-allow)
///
/// ```txt
/// access-param     = permission-group OWS "=" OWS access-modes
/// permission-group = 1*ALPHA
/// access-modes     = DQUOTE OWS *1(access-mode *(RWS access-mode)) OWS DQUOTE
/// access-mode      = "read" / "write" / "append" / "control"
/// ```
///
#[derive(Debug, Clone)]
pub struct AccessParam {
    /// Permission group.
    pub permission_group: PermissionGroup,

    /// Access modes.
    pub access_modes: Vec<AccessMode>,
}

impl AccessParam {
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        buffer.push_str(self.permission_group.as_ref());
        buffer.push('=');
        buffer.push('"');
        self.access_modes.iter().for_each(|am| {
            buffer.push_str(am.as_ref());
            buffer.push(' ');
        });
        buffer.push('"');
    }

    /// Encode the access param target as string
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    /// Decode access-param from encoded value.
    pub fn decode(value: &str) -> Result<Self, InvalidEncodedAccessParam> {
        let (k, v) = value.split_once('=').ok_or(InvalidEncodedAccessParam)?;

        let permission_group =
            PermissionGroup::from_str(k.trim()).map_err(|_| InvalidEncodedAccessParam)?;

        let v = v.trim();
        if !(v.len() >= 2 && v.starts_with(DQUOTE) && v.ends_with(DQUOTE)) {
            return Err(InvalidEncodedAccessParam);
        }

        let access_modes = v[1..v.len() - 1]
            .split_ascii_whitespace()
            .map(AccessMode::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| InvalidEncodedAccessParam)?;

        Ok(Self {
            permission_group,
            access_modes,
        })
    }
}

/// Invalid encoded access param.
#[derive(Debug, thiserror::Error)]
#[error("Invalid encoded access param.")]
pub struct InvalidEncodedAccessParam;

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use rstest::rstest;

    use crate::header::wac_allow::AccessParam;

    #[rstest]
    #[case::invalid_pg("a_b = \"read\"")]
    #[case::invalid_mode("ab = \"reed\"")]
    #[case::no_quotes("ab = read")]
    #[case::csv("ab = \"read, write\"")]
    fn invalid_encoded_param_will_be_rejected(#[case] value: &str) {
        assert_err!(AccessParam::decode(value));
    }

    #[rstest]
    #[case("user =\"\"")]
    #[case("user =\"read   write append \"")]
    #[case("team =\"\"")]
    #[case("public =\"read \"")]
    fn valid_encoded_param_will_round_trip(#[case] value: &str) {
        let param = assert_ok!(
            AccessParam::decode(value),
            "Error in decoding valid encoded value."
        );
        let reencoded = param.str_encode();
        let param2 = assert_ok!(
            AccessParam::decode(&reencoded),
            "Error in decoding re-encoded value."
        );
        assert_eq!(
            param.permission_group, param2.permission_group,
            "roundtripped permission group doesn't match."
        );
        assert_eq!(
            param.access_modes, param2.access_modes,
            "roundtripped access modes doesn't match."
        );
    }
}
