//! I define few utils to aid with rust identifiers iin codegen.
//!

use once_cell::sync::Lazy;

static INVALID_CHARS_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(&format!(
        "[{}]",
        regex::escape(r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##)
    ))
    .unwrap()
});

/// Sanitize the given string to get a valididentifier.
pub fn sanitize_ident(ident: &str) -> String {
    if ident.is_empty() {
        return ident.to_owned();
    }

    // Replace invalid chars with underscore.
    let mut ident = INVALID_CHARS_RE.replace_all(ident, "_").to_string();

    // Ensure first char is not numeric.
    if ident.chars().next().unwrap().is_numeric() {
        ident = format!("N_{}", ident);
    }

    // Ensure it is not a rust keyword.
    if syn::parse_str::<syn::Ident>(&ident).is_err() {
        ident + "_"
    } else {
        ident
    }
}
