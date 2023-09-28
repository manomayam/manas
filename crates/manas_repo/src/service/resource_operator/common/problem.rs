//! I define common problem types for resource operations.

use dyn_problem::define_anon_problem_types;
use typed_record::TypedRecordKey;

define_anon_problem_types!(
    /// Unsupported media type.
    UNSUPPORTED_OPERATION: ("Unsupported operation.");

    /// Access denied.
    ACCESS_DENIED: ("Access denied.");

    /// Preconditions not satisfied.
    PRECONDITIONS_NOT_SATISFIED: ("Preconditions not satisfied.");

    /// Unsupported media type.
    UNSUPPORTED_MEDIA_TYPE: ("Unsupported media type.");

    /// Invalid existing representation state.
    INVALID_EXISTING_REPRESENTATION_STATE: ("Invalid existing representation state.");

    /// Invalid rdf source representation.
    INVALID_RDF_SOURCE_REPRESENTATION: ("Invalid rdf source representation.");

    /// Invalid user supplied containment triples in a container representation.
    INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES: ("Invalid user supplied containment triples in a container representation.");

    /// Invalid user supplied contained resource metadata in a container representation.
    INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA: (
        "Invalid user supplied contained resource metadata in a container representation."
    );

    /// Uri policy violation.
    URI_POLICY_VIOLATION: ("Uri policy violation.");

    /// Payload too large.
    PAYLOAD_TOO_LARGE: ("Payload too large.");

    /// Custom constrain violation.
    CUSTOM_CONSTRAIN_VIOLATION: ("Custom constrain violation.");
);

/// A typed record key for the violated custom constrain.
#[derive(Debug, Clone)]
pub struct KConstrainedBy;

impl TypedRecordKey for KConstrainedBy {
    type Value = Constrain;
}

/// A struct to represent custom constrain.
// TODO improve with id, etc.
#[derive(Debug, Clone)]
pub struct Constrain {
    /// Constrain message.
    pub message: String,
}
