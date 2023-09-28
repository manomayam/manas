//! This module exports handlebars template registry for codegen

use handlebars::Handlebars;
use once_cell::sync::Lazy;

/// spec_mod template id.
pub static SPEC_MOD_TEMPLATE_ID: &str = "SPEC_MOD_TEMPLATE";

/// Template registry.
pub static TEMPLATE_REGISTRY: Lazy<Handlebars> = Lazy::new(|| {
    let mut reg = Handlebars::new();
    reg.register_template_string(SPEC_MOD_TEMPLATE_ID, include_str!("spec_mod.hbs"))
        .unwrap();
    reg
});
