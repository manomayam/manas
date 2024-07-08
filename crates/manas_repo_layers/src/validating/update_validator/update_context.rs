//! I define types to represent rep update contexts.
//!

use std::sync::Arc;

use capped_stream::OutOfSizeLimitError;
use dyn_problem::{type_::UNKNOWN_IO_ERROR, Problem};
use futures::TryFutureExt;
use manas_http::{
    representation::impl_::{
        binary::BinaryRepresentation,
        common::data::{bytes_inmem::BytesInmem, quads_inmem::EcoQuadsInmem},
    },
    BoxError,
};
use manas_repo::{
    service::resource_operator::common::problem::{
        INVALID_RDF_SOURCE_REPRESENTATION, PAYLOAD_TOO_LARGE,
    },
    Repo, RepoExistingResourceToken,
};
use manas_space::resource::slot::SolidResourceSlot;
use rdf_dynsyn::parser::DynSynParserFactorySet;
use tracing::error;
use typed_record::{ClonableTypedRecord, TypedRecordKey};

/// A struct to represent context of representation update.
#[derive(Debug)]
pub struct RepUpdateContext<R: Repo> {
    /// Resource slot.
    pub res_slot: SolidResourceSlot<R::StSpace>,

    /// Repo context.
    pub repo_context: Arc<R::Context>,

    /// Resource's current existing status token.
    /// If it is [`None`], resource is considered not yet existing.
    pub current_res_token: Option<RepoExistingResourceToken<R>>,

    /// resolved new representation.
    pub effective_new_rep: R::Representation,

    /// Operation request extensions.
    /// Used for cache purpose.
    pub op_req_extensions: ClonableTypedRecord,
}

impl<R> RepUpdateContext<R>
where
    R: Repo<Representation = BinaryRepresentation>,
{
    /// Try to get resolved rep quads.
    pub async fn try_resolve_effective_rep_quads(
        mut self,
        parser_factories: Arc<DynSynParserFactorySet>,
        rep_payload_max_size: Option<u64>,
    ) -> Result<(Self, EcoQuadsInmem), Problem> {
        let mut resolved_rep = self.effective_new_rep;

        // Apply payload size limits.
        if let Some(rep_payload_max_size) = rep_payload_max_size {
            resolved_rep = resolved_rep.into_stream_size_capped(rep_payload_max_size);
        }

        // Load representation into memory.
        let resolved_rep_inmem: BinaryRepresentation<BytesInmem> =
            async_convert::TryFrom::try_from(resolved_rep)
                .map_err(|e: BoxError| {
                    error!("Error in loading rep data into memory. {e}");

                    if e.downcast_ref::<OutOfSizeLimitError>().is_some() {
                        PAYLOAD_TOO_LARGE
                            .new_problem_builder()
                            .message(
                                "Representation payload size is greater than configured limit.",
                            )
                            .finish()
                    } else {
                        UNKNOWN_IO_ERROR.new_problem()
                    }
                })
                .await?;

        // Resolve quads.
        let resolved_quads_inmem: EcoQuadsInmem = resolved_rep_inmem
            .try_parse_quads_caching(parser_factories)
            .await
            .ok_or_else(|| {
                error!("Representation is not quads parsable.");
                INVALID_RDF_SOURCE_REPRESENTATION
                    .new_problem_builder()
                    .message("Representation is not quads_parsable.")
                    .finish()
            })?
            .as_ref()
            .map_err(|e| {
                error!("Error in parsing quads from representation. {e}");
                INVALID_RDF_SOURCE_REPRESENTATION.new_problem()
            })?
            .clone();

        self.effective_new_rep = resolved_rep_inmem.into();
        Ok((self, resolved_quads_inmem))
    }
}

/// A typed record key for current representation value.
#[derive(Debug, Clone)]
pub struct KCurrentRepBinaryInmem;

impl TypedRecordKey for KCurrentRepBinaryInmem {
    type Value = BinaryRepresentation<BytesInmem>;
}
