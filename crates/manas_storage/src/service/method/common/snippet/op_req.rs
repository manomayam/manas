//! I provide few common code snippets dealing with repo
//! resource operation requests.
//!

use typed_record::{ClonableTypedRecord, TypedRecordKey};

/// A typed record key for operation request extensions.
#[derive(Debug, Clone)]
pub struct KOpReqExtensions;

impl TypedRecordKey for KOpReqExtensions {
    type Value = ClonableTypedRecord;
}
