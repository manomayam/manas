//! I provide existing-represented resource status token
//! implementation for ODR.
//!

use std::{
    ops::{Bound, Deref},
    str::FromStr,
    sync::Arc,
};

use either::Either;
use futures::TryStreamExt;
use headers::{ContentLength, ContentRange, LastModified};
use if_chain::if_chain;
use itertools::Itertools;
use manas_http::{
    header::{
        common::media_type::{MediaType, TEXT_TURTLE},
        last_modified::LastModifiedExt,
    },
    representation::{
        impl_::{
            basic::BasicRepresentation,
            binary::BinaryRepresentation,
            common::data::{bytes_stream::BytesStream, quads_stream::BoxQuadsStream},
        },
        metadata::{
            derived_etag::DerivedETag, KCompleteContentLength, KContentRange, KContentType,
            KDerivedETag, KLastModified, RepresentationMetadata,
        },
    },
};
use manas_repo::service::resource_operator::{
    common::status_token::{ExistingRepresentedResourceToken, RepoResourceStatusTokenBase},
    reader::rep_preferences::{ContainerRepresentationPreference, RepresentationPreferences},
};
use manas_space::{
    resource::{
        kind::SolidResourceKind, slot::SolidResourceSlot, slot_rel_type::SlotRelationType,
        state::SolidResourceState,
    },
    SolidStorageSpace,
};
use opendal::{ErrorKind, Metadata};
use rdf_dynsyn::{
    correspondence::Correspondent, syntax::invariant::parsable::DynSynParsableSyntax,
};
use rdf_utils::model::term::ArcTerm;
use rdf_vocabularies::ns;
use sophia_api::{prelude::Iri, term::Term};
use tracing::{error, info, warn};
use typed_record::TypedRecord;

use crate::{
    context::ODRContext,
    object_store::{
        object::invariant::ODRFileObjectExt, object_space::assoc::rel_type::sidecar::SidecarRelType,
    },
    resource_context::ODRResourceContext,
    service::resource_operator::common::status_token::inputs::{
        altfm::{AltFatMetadata, AltMetadata},
        container_index::{ODRContainerIndexInputs, CONTAINER_LDP_TYPES},
        supplem_fat_data::SupplemFatDataResolutionError,
        ODRResourceStatusTokenInputs,
    },
    setup::{aux_rep_policy::ODRAuxResourcePolicy, ODRRepresentedResourceState, ODRSetup},
    util::ntriples_serializer::serialize_default_graph_to_ntriples,
    OpendalRepo,
};

/// A struct to represent existing-represented resource status
/// token for odr.
#[derive(Debug, Clone)]
pub struct ODRExistingRepresentedResourceToken<Setup: ODRSetup>(
    pub(in super::super) ODRResourceStatusTokenInputs<Setup>,
);

impl<Setup: ODRSetup> Deref for ODRExistingRepresentedResourceToken<Setup> {
    type Target = ODRResourceStatusTokenInputs<Setup>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Setup: ODRSetup> From<ODRExistingRepresentedResourceToken<Setup>>
    for ODRResourceStatusTokenInputs<Setup>
{
    #[inline]
    fn from(value: ODRExistingRepresentedResourceToken<Setup>) -> Self {
        value.0
    }
}

impl<Setup: ODRSetup> RepoResourceStatusTokenBase for ODRExistingRepresentedResourceToken<Setup> {
    type Repo = OpendalRepo<Setup>;

    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        self.0.res_context.repo_context()
    }
}

impl<Setup: ODRSetup> ExistingRepresentedResourceToken
    for ODRExistingRepresentedResourceToken<Setup>
{
    #[inline]
    fn slot(&self) -> &SolidResourceSlot<Setup::StSpace> {
        self.0.res_context.slot()
    }

    fn rep_validators(&self) -> RepresentationMetadata {
        self.resolve_rep_validators()
    }
}

impl<Setup: ODRSetup> TryFrom<ODRResourceStatusTokenInputs<Setup>>
    for ODRExistingRepresentedResourceToken<Setup>
{
    type Error = String;

    fn try_from(inputs: ODRResourceStatusTokenInputs<Setup>) -> Result<Self, Self::Error> {
        if inputs.slot_path_is_represented && inputs.base_obj_metadata.is_some() {
            Ok(Self(inputs))
        } else {
            Err("Inputs are not that of a represented resource".into())
        }
    }
}

impl<Setup: ODRSetup> ODRExistingRepresentedResourceToken<Setup> {
    /// Get resource status inputs.
    #[inline]
    pub fn status_inputs(&self) -> &ODRResourceStatusTokenInputs<Setup> {
        &self.0
    }

    fn inputs_base_obj_metadata(&self) -> &Metadata {
        self.0
            .base_obj_metadata
            .as_ref()
            .expect("Invariant guarantees it to be some.")
    }

    /// Resolve validators for the representation.
    pub fn resolve_rep_validators(&self) -> RepresentationMetadata {
        let inputs = &self.0;

        // Effective last_modified is maximum of base-object's
        // last-modified and altcontent-object's last-modified.
        let last_modified_dt = [
            inputs.base_obj_metadata.as_ref(),
            inputs.altcontent_obj_metadata.as_ref(),
        ]
        .iter()
        .map(|opt_metadata| opt_metadata.and_then(|metadata| metadata.last_modified()))
        .max()
        .unwrap();

        let last_modified = last_modified_dt.map(LastModified::from_date_time);

        let etag_str = if inputs.res_context.is_left_classified() {
            // For container resources, attach last_modified
            // timestamp too to take index update into account.
            inputs
                .altcontent_obj_metadata
                .as_ref()
                .and_then(|m| m.etag())
                .or_else(|| self.inputs_base_obj_metadata().etag())
                .map(|v| {
                    format!(
                        "{}*{}",
                        v,
                        last_modified_dt
                            .map(|lm| lm.timestamp_millis())
                            .unwrap_or(0)
                    )
                })
        } else {
            self.inputs_base_obj_metadata().etag().map(|v| v.to_owned())
        }
        .or_else(|| last_modified.as_ref().map(|lm| lm.derive_etag()));

        let etag = etag_str.and_then(|v| DerivedETag::try_from(v).ok());

        RepresentationMetadata::new()
            .with_opt::<KLastModified>(last_modified)
            .with_opt::<KDerivedETag>(etag)
    }

    /// Try resolve effective alt metadata.
    pub fn try_resolve_effective_alt_metadata(
        &self,
    ) -> Result<Option<AltMetadata>, SupplemFatDataResolutionError> {
        let inputs = &self.0;

        resolve_effective_alt_metadata(
            if self.res_context.is_left_classified() {
                // If resource is a container, then alt object.
                &inputs.altcontent_obj_metadata
            } else {
                &inputs.base_obj_metadata
            }
            .as_ref(),
            self.altfm_obj_content.as_deref(),
        )
    }

    /// Resolve effective rep content type.
    pub fn resolve_effective_rep_content_type(
        &self,
    ) -> Result<MediaType, ODRResourceStateResolutionError> {
        let inputs = &self.0;

        // Get if backend is content-type meta capable.
        let is_cty_capable_backend = inputs
            .res_context
            .repo_context()
            .object_store
            .is_cty_capable_backend();

        // If backend is cty metadata capable, then that will take precedence.
        if is_cty_capable_backend {
            let opt_cty_str = if self.res_context.is_left_classified() {
                // If resource is a container, then
                if let Some(altcontent_obj_metadata) = &inputs.altcontent_obj_metadata {
                    altcontent_obj_metadata.content_type()
                } else {
                    Some(TEXT_TURTLE.essence_str())
                }
            } else {
                self.inputs_base_obj_metadata().content_type()
            };

            if let Some(cty_str) = opt_cty_str {
                if let Ok(cty) = MediaType::from_str(cty_str).map_err(|e| {
                    warn!("Invalid base object content-type metadata. Err: {}", e);
                    e
                }) {
                    return Ok(cty);
                }
            }
        }
        // Else, If altfm object exists with content_type override, it takes precedence.
        else if let Some(effective_alt_metadata) =
            self.try_resolve_effective_alt_metadata().map_err(|e| {
                error!("Invalid alt meta object.");
                ODRResourceStateResolutionError::EffectiveAltMetadataResolutionError(e)
            })?
        {
            if let Some(alt_content_type) = effective_alt_metadata.content_type {
                return Ok(alt_content_type);
            }
        }

        // Else resolve to default decoded.
        Ok(decode_rep_content_type::<Setup>(&self.res_context))
    }

    /// Try resolve user supplied representation metadata.
    pub fn try_resolve_user_supplied_rep_metadata(
        &self,
    ) -> Result<RepresentationMetadata, ODRResourceStateResolutionError> {
        let inputs = &self.0;

        // Get effective rep content type.
        let effective_rep_content_type =
            self.resolve_effective_rep_content_type().map_err(|e| {
                error!(
                    "Error in resolving effective rep content type. Error:\n {}",
                    e
                );
                e
            })?;

        // Rep complete content length.
        let rep_complete_content_length =
            Some(ContentLength(if inputs.res_context.is_right_classified() {
                self.inputs_base_obj_metadata().content_length()
            } else {
                inputs
                    .altcontent_obj_metadata
                    .as_ref()
                    .map(|m| m.content_length())
                    .unwrap_or_default()
            }));

        // Construct and return rep metadata.
        Ok(self
            .resolve_rep_validators()
            .with::<KContentType>(effective_rep_content_type)
            .with_opt::<KCompleteContentLength>(rep_complete_content_length))
    }

    /// Try resolve representation with given preferences.
    #[allow(clippy::needless_collect)]
    #[tracing::instrument(
        skip_all,
        name = "ODRExistingRepresentedResourceToken::try_resolve_representation",
        fields(rep_preferences)
    )]
    pub async fn try_resolve_representation(
        &self,
        rep_preferences: RepresentationPreferences,
    ) -> Result<BinaryRepresentation, ODRResourceStateResolutionError> {
        let inputs = &self.0;
        let res_context = inputs.res_context.as_ref().as_ref();
        let res_uri = res_context.uri().clone();
        let assoc_obj_map = res_context.assoc_odr_object_map();

        let (resolved_rep_data, resolved_rep_metadata) =
            if let Either::Left(c_res_context) = self.res_context.clone().0 {
                // Resource is a container.

                let c_rep_preference = rep_preferences.container_rep_preference;

                let mut rep_metadata = self.try_resolve_user_supplied_rep_metadata()?;

                // Content type of user supplied rep.
                let us_rep_content_type = rep_metadata
                    .remove_rec_item::<KContentType>()
                    .unwrap_or_default();

                rep_metadata.remove_rec_item::<KCompleteContentLength>();

                let container_term = res_uri.deref().into_term::<ArcTerm>();
                let type_predicate: ArcTerm = ns::rdf::type_.into_term::<ArcTerm>();

                // Construct ldp specified statements about container.
                let mut container_ldp_statements = CONTAINER_LDP_TYPES
                    .iter()
                    .map(|type_| {
                        anyhow::Result::Ok((
                            [
                                container_term.clone(),
                                type_predicate.clone(),
                                type_.into_term(),
                            ],
                            None,
                        ))
                    })
                    .collect::<Vec<_>>();

                // Push any storage related statements.
                if c_res_context.slot().is_root_slot() {
                    container_ldp_statements.push(Ok((
                        [
                            container_term.clone(),
                            type_predicate.clone(),
                            ns::pim::Storage.into_term(),
                        ],
                        None,
                    )));

                    container_ldp_statements.push(Ok((
                        [
                            container_term.clone(),
                            ns::solid::owner.into_term(),
                            c_res_context.slot().space().owner_id().into_term(),
                        ],
                        None,
                    )));
                }

                // Initialize rep with container type statements.
                let mut rep_data: BoxQuadsStream =
                    Box::pin(futures::stream::iter(container_ldp_statements.into_iter()));

                // If preferences require user supplied statements.
                if [
                    ContainerRepresentationPreference::Minimal,
                    ContainerRepresentationPreference::All,
                ]
                .contains(&c_rep_preference)
                {
                    // Resolve rdf doc syntax corresponding to
                    // user supplied rep.
                    let us_rep_syntax = match Correspondent::<DynSynParsableSyntax>::try_from(
                        us_rep_content_type.deref(),
                    ) {
                        Ok(Correspondent { value, is_total }) if is_total => value,
                        _ => {
                            error!("User supplied container rep is not quadable.");
                            return Err(
                                ODRResourceStateResolutionError::InvalidUserSuppliedContainerRep,
                            );
                        }
                    };

                    // Resolve user supplied rep data.
                    let us_rep_data =
                    // If alt-content object exists for 
                    // container, it will contain user supplied 
                    // rep data.
                    if self.altcontent_obj_metadata.is_some() {
                        let content_obj = assoc_obj_map.sidecar_object(SidecarRelType::AltContent);

                        content_obj
                            .stream_complete()
                            .await
                            .map_err(on_content_read_error)?
                    } else {
                        // Else, base object must always be purely namespace object.
                        Box::pin(futures::stream::empty())
                    };

                    let parser_factory_set = res_context
                        .repo_context()
                        .as_ref()
                        .config
                        .dynsyn_factories
                        .as_ref()
                        .parser
                        .clone();

                    // Parse quads from user supplied rep.
                    let us_rep_data_quads = parser_factory_set
                        .as_ref()
                        .parse_quads_from_bytes_stream::<_, ArcTerm>(
                            us_rep_data,
                            Some(Iri::new_unchecked(res_uri.as_str().to_owned())),
                            us_rep_syntax,
                        )
                        .await
                        .map_err(anyhow::Error::new);

                    // Include user supplied statements to effective rep data.
                    rep_data = Box::pin(futures::stream::select(rep_data, us_rep_data_quads));
                }

                // If preferences requests for containment triples.
                if [
                    ContainerRepresentationPreference::Containment,
                    ContainerRepresentationPreference::All,
                ]
                .contains(&c_rep_preference)
                {
                    // Resolve container index quads.
                    let container_index_quads = ODRContainerIndexInputs { c_res_context }
                        .resolve()
                        .await
                        .map_err(|e| {
                            error!("Io error in resolving container index. Error:\n {}", e);
                            ODRResourceStateResolutionError::UnknownIoError(e)
                        })?;

                    // Include containment statements and
                    // contained metadata statements in effective rep.
                    rep_data = Box::pin(futures::stream::select(rep_data, container_index_quads));
                }

                // Resolve.
                (
                    serialize_default_graph_to_ntriples(rep_data),
                    // Set content type to quads.
                    rep_metadata.with::<KContentType>((*TEXT_TURTLE).clone()),
                )
            } else {
                //Resource is a non-container.
                let base_obj = assoc_obj_map
                    .base_object()
                    .as_right_classified()
                    .expect("Base object must be file object for non containers.");

                // Resolve metadata of the complete representation.
                let rep_metadata = self.try_resolve_user_supplied_rep_metadata()?;

                let (rep_data, content_range) = if_chain! {
                    // If backend can support range request.
                    if res_context.repo_context().backend_caps().read_with_range;

                    // Collect all requested sub ranges, if it is range request.
                    if let Some(req_sub_range) = rep_preferences
                        .non_container_rep_range_negotiator
                        .resolve_pref_range(&rep_metadata)
                    // ODR supports only single range requests.
                        .and_then(|r| r.iter().exactly_one().ok());

                    // If content range is resolvable from requested range,
                    if let Ok(content_range) = ContentRange::bytes(
                        req_sub_range,
                        Some(self.inputs_base_obj_metadata().content_length())
                    )
                        .map_err(|e| {
                            info!("Invalid content range");
                            e
                        });

                    // And if bytes range bounds can be resolved.
                    if let Some(bytes_range_bounds) =content_range.bytes_range().map(
                        |(s, e)| (Bound::Included(s), Bound::Included(e))
                    );

                    then {
                        // Then satisfy range request with partial rep data.
                        (
                            base_obj.stream_range(bytes_range_bounds).await
                                .map_err(on_content_read_error)?,
                            Some(content_range),
                        )
                    } else {
                        // Else with complete rep data.
                        (
                            base_obj.stream_complete().await
                                .map_err(on_content_read_error)?,
                            None,
                        )
                    }
                };

                (
                    rep_data,
                    // Resolve metadata with resolved content range.
                    rep_metadata.with_opt::<KContentRange>(content_range),
                )
            };

        Ok(BasicRepresentation {
            metadata: resolved_rep_metadata,
            data: BytesStream {
                stream: resolved_rep_data,
                size_hint: Default::default(),
            },
            base_uri: Some(res_uri.into_subject()),
        }
        .into_binary())
    }

    /// Try resolve resource state.
    pub async fn try_resolve_resource_state(
        &self,
        rep_preferences: RepresentationPreferences,
    ) -> Result<ODRRepresentedResourceState<Setup>, ODRResourceStateResolutionError> {
        // Resolve representation.
        let representation = self.try_resolve_representation(rep_preferences).await?;

        Ok(SolidResourceState {
            slot: self.0.res_context.slot().clone(),
            representation: Some(representation),
            // aux_links: self.0.res_context.supported_aux_links().collect(),
        }
        .try_into()
        .expect("Must be represented."))
    }
}

/// An error type for errors in resolving resource state for ODR.
#[derive(Debug, thiserror::Error)]
pub enum ODRResourceStateResolutionError {
    /// Effective alt metadata resolution error.
    #[error("Effective alt metadata resolution error.")]
    EffectiveAltMetadataResolutionError(#[from] SupplemFatDataResolutionError),

    /// Invalid user supplied container representation.
    #[error("Invalid user supplied container representation.")]
    InvalidUserSuppliedContainerRep,

    /// Unknown io error.
    #[error("Unknown io error.")]
    UnknownIoError(opendal::Error),
}

/// Resolve effective alt metadata from given base obj metadata
/// and optional altfm obj content.
fn resolve_effective_alt_metadata(
    opt_content_obj_metadata: Option<&Metadata>,
    opt_altfm_obj_content: Option<&[u8]>,
) -> Result<Option<AltMetadata>, SupplemFatDataResolutionError> {
    opt_content_obj_metadata
        .and_then(|content_obj_metadata| {
            opt_altfm_obj_content.map(|altfm_obj_content| {
                serde_json::from_slice::<AltFatMetadata>(altfm_obj_content)
                    .map_err(|_| {
                        error!("Invalid altfm object content");
                        SupplemFatDataResolutionError::InvalidSupplemFatData
                    })
                    .and_then(|altfm| altfm.resolve_effective_supplem_data(content_obj_metadata))
            })
        })
        .unwrap_or(Ok(None))
}

/// Decodes representation's default content type from resource slot.
/// It follows following steps in resolution.
///
/// 1. If res is a container, it resolves to `text/turtle`.
///
/// 2. If res is aux resource, it resolves from setup config.
///
/// 3. Otherwise it tries to mime-guess from resource uri path.
///
/// 4. If not resolved, fallbacks to `application/octet-stream`.
///
pub(crate) fn decode_rep_content_type<Setup: ODRSetup>(
    res_context: &ODRResourceContext<Setup>,
) -> MediaType {
    // If res is a container, return turtle.
    if res_context.kind() == SolidResourceKind::Container {
        return (*TEXT_TURTLE).clone();
    }

    // If is rep object of aux resource, then return as per odr config.
    if_chain!(
        if let Some(last_rev_link) = res_context.slot().slot_rev_link();
        if let SlotRelationType::Auxiliary(aux_rel_type) = &last_rev_link.rev_rel_type;

        then {
            return <Setup::AuxResourcePolicy as ODRAuxResourcePolicy>::aux_rep_media_type(aux_rel_type).clone()
        }
    );

    // Otherwise guess from uri.
    MediaType::guess_from_path(res_context.uri().as_ref().inner().path_str()).unwrap_or_default()
}

/// Callback function to handle error in reading content of an
/// existing object.
fn on_content_read_error(e: opendal::Error) -> ODRResourceStateResolutionError {
    match e.kind() {
        // If not found as promised, then warn.
        ErrorKind::NotFound => {
            warn!("Object doesn't exist in backend as promised.");
        }
        _ => {
            error!(
                "Unknown io error in dereferencing object content stream. Error: {:?}",
                e
            );
        }
    }
    ODRResourceStateResolutionError::UnknownIoError(e)
}
