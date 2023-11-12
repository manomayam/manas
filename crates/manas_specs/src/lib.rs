//! This crate provides types to represent spec requirements, and exports statics for different specs in solid ecosystem.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use dyn_problem::{ProblemBuilder, ProblemBuilderExt};
use http::StatusCode;
use http_api_problem::{ApiError, ApiErrorBuilder};
use iri_string::types::{UriReferenceString, UriStr, UriString};
use manas_http::uri::component::character::pchar::PATHCHAR_PCT_ENCODE_SET;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_record::TypedRecordKey;

pub mod protocol;

/// Requirement Levels as specified in [`RFC2119`](https://datatracker.ietf.org/doc/html/rfc2119)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementLevel {
    /// MUST
    Must,

    /// MUST_NOT
    MustNot,

    /// REQUIRED
    Required,

    /// SHALL
    Shall,

    /// SHALL_NOT
    ShallNot,

    /// SHOULD
    Should,

    ///SHOULD_NOT
    ShouldNot,

    /// RECOMMENDED
    Recommended,

    /// MAY
    May,

    /// OPTIONAL
    Optional,
}

/// Represents a specification requirement
#[derive(Debug, Clone)]
pub struct Requirement<Spec>
where
    Spec: Specification,
{
    /// Uri of the requirement.
    pub id: Cow<'static, UriStr>,

    /// Subjects of the requirement.,
    pub subjects: Cow<'static, [UriString]>,

    /// Requirement level.
    pub level: RequirementLevel,

    /// Requirement statement.
    pub statement: &'static str,

    _phantom: PhantomData<Spec>,
}

impl<Spec> Requirement<Spec>
where
    Spec: Specification,
{
    /// Create a new [`Requirement`] with given params.
    #[inline]
    pub fn new(
        id: Cow<'static, UriStr>,
        subjects: Cow<'static, [UriString]>,
        level: RequirementLevel,
        statement: &'static str,
    ) -> Self {
        Self {
            id,
            subjects,
            level,
            statement,
            _phantom: PhantomData,
        }
    }
}

/// A minimal view of [`Requirement`].
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementMinView {
    /// Uri of the requirement.
    pub id: Cow<'static, UriStr>,

    /// Requirement statement.
    pub statement: &'static str,
}

impl<Spec: Specification> From<Requirement<Spec>> for RequirementMinView {
    #[inline]
    fn from(r: Requirement<Spec>) -> Self {
        Self {
            id: r.id,
            statement: r.statement,
        }
    }
}

/// A trait for defining a specification type.
pub trait Specification: Send + Sync + Debug + Clone + 'static {
    /// Get uri of specification.
    fn uri() -> &'static UriStr;

    /// Get title of specification.
    fn title() -> &'static str;
}

/// Struct representing a spec problem.
#[derive(Debug, Clone, thiserror::Error)]
pub struct SpecProblem<Spec>
where
    Spec: Specification,
{
    /// Status code for problem.
    pub status_code: StatusCode,

    /// Violated requirement.
    pub violated: Option<&'static Requirement<Spec>>,

    /// Specified recourse requirement.
    pub recourse_as_per: Option<&'static Requirement<Spec>>,

    /// Spec problem detail.
    pub spec_problem_detail: Option<String>,

    /// A URI reference that identifies the specific
    /// occurrence of the problem.  It may or may not yield further
    /// information if dereferenced.
    pub instance: Option<UriReferenceString>,
}

impl<Spec: Specification> SpecProblem<Spec> {
    /// Get new spec problem with given status code.
    #[inline]
    pub fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            violated: None,
            recourse_as_per: None,
            spec_problem_detail: None,
            instance: None,
        }
    }

    /// With violated.
    #[inline]
    pub fn with_violated(mut self, violated: &'static Requirement<Spec>) -> Self {
        self.violated = Some(violated);
        self
    }

    /// With recourse as per.
    #[inline]
    pub fn with_recourse_as_per(mut self, recourse_as_per: &'static Requirement<Spec>) -> Self {
        self.recourse_as_per = Some(recourse_as_per);
        self
    }

    /// With spec problem detail.
    #[inline]
    pub fn with_spec_problem_detail(mut self, violation_detail: &str) -> Self {
        self.spec_problem_detail = Some(violation_detail.to_owned());
        self
    }

    /// With instance.
    #[inline]
    pub fn with_instance(mut self, instance: UriReferenceString) -> Self {
        self.instance = Some(instance);
        self
    }
}

/// Namespace base uri for spec error problem types.
pub static SPEC_PROBLEM_TYPE_NS_BASE: &str = "urn::manas::spec_problem#";

/// A [`TypedRecordKey`] for violated requirement.
#[derive(Debug, Clone)]
pub struct KViolatedReq<Spec: Specification> {
    _phantom: PhantomData<Spec>,
}

impl<Spec: Specification> TypedRecordKey for KViolatedReq<Spec> {
    type Value = &'static Requirement<Spec>;
}

/// A [`TypedRecordKey`] for recourse_as_per requirement.
#[derive(Debug, Clone)]
pub struct KRecourseAsPerReq<Spec: Specification> {
    _phantom: PhantomData<Spec>,
}

impl<Spec: Specification> TypedRecordKey for KRecourseAsPerReq<Spec> {
    type Value = &'static Requirement<Spec>;
}

impl<Spec: Specification> From<SpecProblem<Spec>> for ApiError {
    #[inline]
    fn from(val: SpecProblem<Spec>) -> Self {
        ApiErrorBuilder::from(val).finish()
    }
}

impl<Spec: Specification> From<SpecProblem<Spec>> for ProblemBuilder {
    #[inline]
    fn from(val: SpecProblem<Spec>) -> Self {
        ApiErrorBuilder::from(val).into()
    }
}

impl<Spec: Specification> From<SpecProblem<Spec>> for ApiErrorBuilder {
    fn from(val: SpecProblem<Spec>) -> Self {
        let mut builder = ApiError::builder(val.status_code)
            .type_url(format!(
                "{}{}",
                SPEC_PROBLEM_TYPE_NS_BASE,
                percent_encoding::utf8_percent_encode(
                    Spec::uri().as_str(),
                    PATHCHAR_PCT_ENCODE_SET
                ),
            ))
            // Derive title from spec title.
            .title(format!("{} specification error.", Spec::title()))
            // Derive message from violated/recourse requirements.
            .message(
                val.violated
                    .map(|violated| format!("Violated: {}", violated.statement))
                    .or_else(|| {
                        val.recourse_as_per.map(|recourse_as_per| {
                            format!("Recourse as per: {}", recourse_as_per.statement)
                        })
                    })
                    .unwrap_or_else(|| "Unknown spec violation.".to_owned()),
            )
            // Attach violated, recourse requirements as extensions and fields.
            .extend_with_opt::<KViolatedReq<Spec>>(val.violated)
            .field(
                "violated",
                val.violated
                    .map(|violated| RequirementMinView::from(violated.clone())),
            )
            .extend_with_opt::<KRecourseAsPerReq<Spec>>(val.recourse_as_per)
            .field(
                "recourse_as_per",
                val.recourse_as_per
                    .map(|recourse_as_per| RequirementMinView::from(recourse_as_per.clone())),
            );

        // Set instance uri.
        if let Some(instance) = val.instance {
            builder = builder.instance(instance.as_str());
        }

        // Attach any spec error detail.
        if let Some(spec_err_detail) = val.spec_problem_detail {
            builder = builder.field("violation_detail", spec_err_detail);
        }

        builder
    }
}

/// A macro for defining requirement
#[macro_export(local_inner_macros)]
macro_rules! spec_mod {
    (
        $(#[$SPEC_OUTER:meta])*
        Spec: (
            $SPEC_ID: ident,
            $SPEC_TITLE: expr,
            $SPEC_URI: expr
        );

        Subjects: [
            $(
                $(#[$SUB_OUTER:meta])*
                (
                    $SUB_ID: ident,
                    $SUB_URI: expr
                ),
            )+
        ];

        Requirements: [
            $(
                $(#[$REQ_OUTER:meta])*
                (
                    $REQ_ID: ident,
                    $REQ_URI: expr,
                    $REQ_LEVEL: expr,
                    [
                        $($REQ_SUB: expr,)*
                    ],
                    $REQ_STATEMENT: expr
                ),
            )+
        ];
    ) => {
        use std::borrow::Cow;

        use iri_string::types::UriStr;
        use once_cell::sync::Lazy;

        use $crate::{Requirement, RequirementLevel, Specification};

        $(#[$SPEC_OUTER])*
        #[derive(Debug, Clone)]
        pub struct $SPEC_ID;

        impl Specification for $SPEC_ID {
            #[inline]
            fn uri() -> &'static UriStr {
                &SPEC_URI
            }

            #[inline]
            fn title() -> &'static str {
                $SPEC_TITLE
            }
        }

        /// URI of the spec.
        static SPEC_URI: Lazy<&'static UriStr> = Lazy::new(|| {
            $SPEC_URI
                .try_into()
                .expect("Must be valid uri.")
        });

        $(
            $(#[$SUB_OUTER])*
            #[allow(missing_docs)]
            pub static $SUB_ID: Lazy<&UriStr> = Lazy::new(|| {
                $SUB_URI
                    .try_into()
                    .expect("Must be valid uri.")
            });
        )*

        $(
            $(#[$REQ_OUTER])*
            #[allow(missing_docs)]
            pub static $REQ_ID: Lazy<Requirement<$SPEC_ID>> = Lazy::new(|| {
                Requirement::new(
                    Cow::Borrowed(
                        $REQ_URI
                            .try_into()
                            .expect("Must be valid uri."),
                    ),
                    Cow::Owned(std::vec![
                        $($REQ_SUB.to_owned())*
                    ]),
                    $REQ_LEVEL,
                    $REQ_STATEMENT,
                )
            });
        )*

        mod tests {
            #[test]
            fn spec_uri_is_valid() {
                let _= &*super::SPEC_URI;
            }

            #[test]
            fn test_subject_names() {
                $(
                    let _ = &*super::$SUB_ID;
                )*
            }

            #[test]
            fn test_requirements() {
                $(
                    let _ = &*super::$REQ_ID;
                )*
            }
        }
    };
}
