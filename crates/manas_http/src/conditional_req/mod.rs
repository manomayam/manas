//! I define utilities to handle http conditional requests.
//!

//! I define helper functions to handle preconditions
//! in conditional http requests, as per [`rfc9110`](https://www.rfc-editor.org/rfc/rfc9110.html#name-conditional-requests).
//!

use headers::{
    ETag, Header, HeaderMap, HeaderMapExt, IfMatch, IfModifiedSince, IfNoneMatch,
    IfUnmodifiedSince, LastModified,
};
use http::{Method, StatusCode};
use if_chain::if_chain;

/// [`PreconditionsEvaluator`] evaluates http request preconditions
/// following algorithm specified at [rfc9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-precedence-of-preconditions).
///
pub struct PreconditionsEvaluator<'p> {
    /// Request method.
    pub req_method: Method,

    /// Request headers.
    pub req_headers: &'p HeaderMap,

    /// If resource is represented.
    pub res_is_represented: bool,

    /// Resource last modified time.
    pub res_last_modified: Option<&'p LastModified>,

    /// Selected representation etag.
    pub selected_rep_etag: Option<&'p ETag>,
}

impl<'p> PreconditionsEvaluator<'p> {
    /// Evaluate preconditions.
    pub fn evaluate(&self) -> PreconditionsResolvedAction {
        self.step1()
            .or_else(|| {
                self.step2().or_else(|| {
                    self.step3()
                        .or_else(|| self.step4().or_else(|| self.step5()))
                })
            })
            .unwrap_or_else(|| self.step6())
    }

    /// Get typed header from request headers.
    #[inline]
    fn h<H: Header>(&self) -> Option<H> {
        self.req_headers.typed_get()
    }

    /// Step1:
    ///
    /// When recipient is the origin server and If-Match is present, evaluate the If-Match precondition:¶
    /// - if true, continue to step 3
    /// - if false, respond 412 (Precondition Failed) unless it can be determined that the state-changing request has already succeeded (see Section 13.1.1)
    fn step1(&self) -> Option<PreconditionsResolvedAction> {
        self.h::<IfMatch>().and_then(|if_match| {
            let passes = self
                .selected_rep_etag
                .map(|etag| if_match.precondition_passes(etag))
                .unwrap_or(false);
            if passes {
                self.step3()
            } else {
                Some(PreconditionsResolvedAction::Return(
                    StatusCode::PRECONDITION_FAILED,
                ))
            }
        })
    }

    /// Step2:
    ///
    /// When recipient is the origin server, If-Match is not present, and If-Unmodified-Since is present, evaluate the If-Unmodified-Since precondition:
    /// - if true, continue to step 3
    /// - if false, respond 412 (Precondition Failed) unless it can be determined that the state-changing request has already succeeded (see Section 13.1.4)
    fn step2(&self) -> Option<PreconditionsResolvedAction> {
        if_chain! {
            if let Some(if_unmodified_since) = self.h::<IfUnmodifiedSince>();
            if let Some(res_last_modified) = self.res_last_modified;
            if self.h::<IfMatch>().is_none();

            then {
                let passes = if_unmodified_since
                    .precondition_passes((*res_last_modified).into());
                if passes {
                    self.step3()
                } else {
                    Some(PreconditionsResolvedAction::Return(StatusCode::PRECONDITION_FAILED))
                }
            }
            else {None}
        }
    }

    /// Step3:
    ///
    /// When If-None-Match is present, evaluate the If-None-Match precondition:
    /// - if true, continue to step 5
    /// - if false for GET/HEAD, respond 304 (Not Modified)
    /// - if false for other methods, respond 412 (Precondition Failed)
    fn step3(&self) -> Option<PreconditionsResolvedAction> {
        self.h::<IfNoneMatch>().and_then(|if_none_match| {
            let passes = if (if_none_match == IfNoneMatch::any()) && self.res_is_represented {
                false
            } else {
                self.selected_rep_etag
                    .map(|etag| if_none_match.precondition_passes(etag))
                    .unwrap_or(true)
            };

            if passes {
                self.step5()
            } else {
                Some(PreconditionsResolvedAction::Return(
                    if [Method::GET, Method::HEAD].contains(&self.req_method) {
                        StatusCode::NOT_MODIFIED
                    } else {
                        StatusCode::PRECONDITION_FAILED
                    },
                ))
            }
        })
    }

    /// Step4:
    ///
    /// When the method is GET or HEAD, If-None-Match is not present, and If-Modified-Since is present, evaluate the If-Modified-Since precondition:
    /// - if true, continue to step 5
    /// - if false, respond 304 (Not Modified)
    fn step4(&self) -> Option<PreconditionsResolvedAction> {
        if_chain! {
           if  [Method::GET, Method::HEAD].contains(&self.req_method);
            if let Some(if_modified_since) = self.h::<IfModifiedSince>();
            if let Some(res_last_modified) = self.res_last_modified;
            if self.h::<IfNoneMatch>().is_none();

            then {
                let passes = if_modified_since.is_modified((*res_last_modified).into());
                if passes {
                    self.step5()
                } else {
                    Some(PreconditionsResolvedAction::Return(StatusCode::NOT_MODIFIED))
                }
            }

            else {
                None
            }
        }
    }

    /// Step5:
    ///
    /// When the method is GET and both Range and If-Range are present, evaluate the If-Range precondition:
    /// - if true and the Range is applicable to the selected representation, respond 206 (Partial Content)
    /// - otherwise, ignore the Range header field and respond 200 (OK)
    #[inline]
    fn step5(&self) -> Option<PreconditionsResolvedAction> {
        if [Method::GET].contains(&self.req_method) {
            Some(PreconditionsResolvedAction::ApplyMethod)
        } else {
            None
        }
    }

    /// Step6:
    ///
    /// Otherwise,
    /// perform the requested method and respond according to its success or failure.
    #[inline]
    fn step6(&self) -> PreconditionsResolvedAction {
        PreconditionsResolvedAction::ApplyMethod
    }
}

/// Resolved action after evaluation of preconditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreconditionsResolvedAction {
    /// Apply request method.
    ApplyMethod,

    /// Return response with given status code.
    Return(StatusCode),
}

impl PreconditionsResolvedAction {
    /// Get the value if it is `Return` variant.
    pub fn as_return(&self) -> Option<&StatusCode> {
        match self {
            Self::ApplyMethod => None,
            Self::Return(c) => Some(c),
        }
    }
}
