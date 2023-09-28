# Architecture

[Solid protocol](https://solidproject.org/TR/protocol) document enumerates the set of requirements solid servers and clients must satisfy. But for implementations, it is necessary to extract essential model and abstractions that spread across those requirements, and formulate clear architecture that satisfies desirable system properties.

This document provides an overview of manas's architecture, and how it relates to protocol's requirements and the set of desired properties mentioned in the introduction.

-------------------------------------

Solid server is a specialization of `Origin server` component in web architecture. It organizes it's world into information resources, each uniquely identifiable by a URI,and provides a general interface to access and mutate their representations following REST principles. It uses http as the transfer protocol.

The specialization comes from the extra constraints and extensions it adds to resource naming, resource relations, and http method semantics.

> **ℹ️** Integrating with hyper's eco-system, Manas project provides [`manas_http`] crate, that provides
> * Types for various invariants of http uris
> * Exhaustive set of well tested typed headers, (`Link`, `Accept`, `Accept-<Method>`, `WWW-Authenticate`, `Wac-Allow`, etc.) which helps great in headerful ecosystem of solid.
> * Algorithms for conditional requests, as specified in rfc9110.
> * Fundamental trait for representations, that is used through out Manas ecosystem.
> * Implementations of binary representations and quads representations, and many conversions between them.
> * Fundamental trait for http services as a sub trait of tower's `Service` trait, and few utility layers to reconstructing uri, normalizing uri, routing-by-method, etc.
>
> It is both server and client usable.
> 

## Resource space

The solid server's resource space is organized into one or more non-overlapping  `Storage spaces`. It is server's responsibility to protect the invariants of these storage spaces.

Each storage space is characterized by a unique storage root resource of type `pim:Storage`. It must have one or more owners (with relation `solid:owner`), and a description resource (with relation `solid:storageDescription`).

> **ℹ️** [`manas_space`] crate provides rust models for abstractions that together define solid resource space. 

> **ℹ️** [`manas_space::SolidStorageSpace`] trait defines interface to declare storage spaces, their properties and policies.

All resources in a storage space have type `ldp:Resource`. They can be either rdf sources (LDP-RS), or non rdf resources (LDP-NR).

### Resource uri

Each resource in the resource space has a unique and opaque http absolute uri as it's name.

As resource_uri has to be de-referable through http, and as http specifies uri normalization, there arises problem when integrating with rdf.

As rdf requires literal comparison of the names, and http recommending normalization (which any intermediates can apply), Manas constrains all the solid resource uris to be normal-absolute http uris.

> **ℹ️** [`manas_space::resource::uri::SolidResourceUri`] models the correct invariant for solid resource uris.
> ```rust
>   /// A resource uri is an http absolute uri in normal form.
>   pub type SolidResourceUri = NormalAbsoluteHttpUri;
>   pub type NormalAbsoluteHttpUri = Proven<HttpUri, AllOf<HttpUri, HList!(IsNormal, IsAbsolute)>>;
> ```

> **⚠️** Note that, solid protocol also includes a requirement to overload slash-semantics to resource uris. Though this requirement adds familiar affordances with relative uris on the web, it doesn't add any core functionality to the system.
>
> While developing the Manas, it was observed that, these overloaded semantics corrupt the proper perception of the domain, and pollutes the architecture with ad-hoc string operations on uris, hiding many architectural flaws.
>
> Thus the guaranteed-assumption of uri semantics is forbidden in Manas codebase. Any navigation must be based on explicit links and relations.
> 
> At the same time Manas architecture provides hooks for repo implementations to enforce their chosen uri policy. And [`manas_semslot`] crate provides extensive type driven abstractions to encode/decode slot semantics to/from resource uri. It can be used in repo implementations that want to enforce/assume certain uri policy.

### Resource slot id

Each resource has a unique slot in the resource space, identified by a slot_id.

A slot_id is nothing but a product of resource_uri and a reference to it's storage_space. This abstraction allows to carry around resource's storage_space context along with it's uri.

> **ℹ️** [`manas_space::resource::slot_id::SolidResourceSlotId`]
> ```rust
> /// A [`SolidResourceSlotId`] is a product of resource uri, and
> /// a link to the storage space it is part of.
> #[derive(Clone, PartialEq, Eq)]
> pub struct SolidResourceSlotId<Space>
> where
>     Space: SolidStorageSpace,
> {
>     /// Provenience storage space of this resource.
>     pub space: Arc<Space>,
> 
>     /// Uri of the resource.
>     pub uri: SolidResourceUri,
> }
>```


### Resource kind

By their mereological function resources are classified in two kinds. Some are containers with type `ldp:BasicContainer`, and some or not (non-containers). Note that their mereologic kind is an absolute property of resources themselves.

> **ℹ️** [`manas_space::resource::kind::ResourceKind`] enum encodes the mereologic kind of resources.
> ```rust
> /// An enum representing kind of resources in a solid storage space.
> #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
> pub enum SolidResourceKind {
>     /// Container resource kind.
>     Container,
> 
>     /// Non container resource kind.
>     NonContainer,
> }
> ```
>

Storage root resource is required to be a container.

### Resource relations

Though resources can have any number of relations among them (May be encoded in their descriptions or representations), Solid specifies few relations and specifies extra constraints and affordances for them.

`solid:storageDescription` type relation connects storage to it's description resource.

Along with above, there are relations that expresses relations between resources in the storage space layout. These are managed by the server. In Manas, they are termed as slot relations.

There are two sets of layout slot_relations. One singleton-set to describe containment, and other set for any auxiliary relations.

In concrete:

1. `ldp:contains` relation relates a container resource to one of it's contained resource.

    Only containers (With `SolidResourceKind::Container` kind, analogous to `ldp:BasicContainer` type) can be subjects of this relation. Target can be any.

2. Any other specified auxiliary relations, relating a subject resource to it's auxiliary resources. Protocol specifies `acl` and `describedby` auxiliary relation types. They can be extended.

While containment relations together arranges resources in a hierarchical tree layout, auxiliary relations augments resources with their auxiliary resources, aside from the tree layout.

These layout relations are inverse-functional, and mutually exclusive. I.e. a resource can be either a contained resource or an auxiliary resource of a unique host resource.

Orphan resources are not allowed. Except storage root, every resource must have a unique inverse layout relation (containment/aux) to it's host (parent-container/aux-subject) resource.

> **ℹ️** [`manas_space::resource::slot::SlotRelType`] ADT models the layout slot relation types.
> ```rust
> #[derive(Debug, Clone, PartialEq, Eq, Hash)]
> pub enum SlotRelationType<KnAux> {
>     /// Contains relation type
>     Contains,
> 
>     /// Auxiliary relation type.
>     Auxiliary(KnAux),
> }
>`````

> **ℹ️** Note that for every solid storage space [`SolidStorageSpace`], it provides an [`AuxPolicy`] that specifies the all known-aux relation types for that space and their characteristics.


### Resource slot links

A `slot_link` is a link from a resource to one of it's hosted (contained/aux) resource, through one of above mentioned  layout slot_relations.

A `slot_rev_link` is a reverse link from a resource to it's optional(in the case of storage root) host resource, through one of above mentioned layout slot_relations.

> **ℹ️** [`manas_space::resource::slot_link::SlotLink`] models the resource slot links.
> ```rust
> pub struct SlotLink<Space>
> where
>     Space: SolidStorageSpace,
> {
>     /// Target of link.
>     pub target: SolidResourceUri,
> 
>     /// rel type.
>     pub rel_type: SpcSlotRelType<Space>,
> }
> ```

> **ℹ️** [`manas_space::resource::slot_link::SlotRevLink`] models the resource slot reverse links
> ```rust
> /// A struct representing a slot reverse link from a
> /// resource to it's immediate host resource.
> #[derive(Debug, Clone, PartialEq, Eq)]
> pub struct SlotRevLink<Space>
> where
>     Space: SolidStorageSpace,
> {
>     /// Target of link.
>     pub target: SolidResourceUri,
>
>     /// Reverse link rel type.
>     pub rev_rel_type: SlotRelationType<SpcKnownAuxRelType<Space>>,
> }
>`````

### Resource slot

Each resource *existing* in the resource space is characterized by unique resource slot.

A resource `slot` is the product of resource's `slot_id`, it's `kind`, and it's functional `slot_rev_link` (None for storage root).

This abstraction is important, as it is the quantum of the storage_space's layout information, that is distributed to each resource. I.e. by summing the information about slots of all the resources, we get the complete layout information.

As we will see, the slot information is exactly what protocol mandates to pass on with resource requests as metadata in headers in alignment with REST's HATEOS.

> **ℹ️** [`manas_space::resource::slot::SolidResourceSlot`] struct models the resource slot.
> 
> ```rust
> /// A resource in a solid storage space has a unique slot
> /// characterized by the product of resource slot id, resource
> /// kind, and it's slot reverse link.
> #[derive(Debug, Clone, PartialEq)]
> pub struct SolidResourceSlot<Space>
> where
>     Space: SolidStorageSpace,
> {
>     /// Slot id.
>     id: SolidResourceSlotId<Space>,
> 
>     /// Kind of the resource.
>     res_kind: SolidResourceKind,
> 
>     /// Slot reverse link of the resource.
>     slot_rev_link: Option<SlotRevLink<Space>>,
> }
> ```

### Resource representation

For each resource existing in the resource space, server associates an optional representation set.

From rfc9110:

> A "representation" is information that is intended to reflect
> a past, current, or desired state of a given resource,
> in a format that can be readily communicated via the protocol.
> A representation consists of a set of representation metadata and
> a potentially unbounded stream of representation data

For rdf-source resources (LDP-RS), the representation can be in any media type that can encode rdf graphs.

For non-rdf-sources (LDP-NR), representation can be in any media type.

Solid protocol though adds following constraints.

* All rdf-source resources must have representations in both  `text/turtle`, and `application/ld+json` media types (May be derived views at run time).
* It is recommended to have self-describing resources.
* For container resources:
    * They are rdf-sources.
    * Their representation contains valid set of containment statements reflecting their containment index.
        As in `<c1/> ldp:contains <c1/r1>.`
    * They must also have [specified minimal metadata](https://solidproject.org/ED/protocol#contained-resource-metadata) statements describing the contained resources. 
    * Both containment statements and contained-resource-metadata statements are server protected, and it is server's responsibility to ensure their correspondence with ground truth.
    * The `Last-Modified` of a container representation will only reflect changes in containment triples, and not others. Provided rational is to avoid propagation of changes up the hierarchy, when contained resource metadata changes.
* For storage root description resource:
    * Must include [protocol required](https://solidproject.org/ED/protocol#storage-description-statements) description statements about storage.
* Auxiliary resources must be rdf sources, and should have compatible representations.
    > **⚠️** Note that Manas relaxes this restriction. Auxiliary resources can be rdf-source/non-rdf-source resources. They can also be containers. Each [`SolidStorageSpace`] configures it's [`AuxPolicy`], where it specifies further constraints.

> **ℹ️** [`manas_http::representation::Representation`] trait defines abstract interface for representations in Manas ecosystem. [`manas_http::representation::impl_::basic::BasicRepresentation`], and [`manas_http::representation::impl_::binary::BinaryRepresentation`] provides concrete support with emphasized support for binary, and rdf representations.

### Resource state

For each resource *existing* in the resource space, it's state is conveyed as a product of it's `slot`, and optionally one of it's `representation`s. The representation itself is the product of representation metadata and data.

Resource is allowed to exist without any representations.

> **ℹ️** [`manas_space::resource::state::SolidResourceState`] models the resource state.
> ```rust
> /// A struct for representing the state of a solid resource.
> #[derive(Debug, Clone)]
> pub struct SolidResourceState<Space, Rep>
> where
>     Space: SolidStorageSpace,
> {
>     /// Slot of the resource.
>     pub slot: SolidResourceSlot<Space>,
> 
>     /// Optional representation of the resource.
>     pub representation: Option<Rep>,
> }
> ```


> **ℹ️** Note that, `manas_space` crate provides many invariants, conversions, methods, and required support to effectively use these and other abstractions related to server's resource space.


## Interaction model

As the solid server is a specialization of http origin server component, it's connector also provide a generic interface for accessing and manipulating the representation set of a resource.

Following REST principles, each server-client interaction is stateless, and carried through self contained request/response messages, using resource uri to specify request target, http method as the primary signifier of interaction semantics, and headers as modifiers of request semantics.

The method semantics are primarily specified in the protocol section [5. Reading and Writing Resources](https://solidproject.org/ED/protocol#reading-writing-resources).

Few general constraints Solid added over rfc9110 are:

* Support for conditional requests made mandatory.
* Uri slash semantics are part of various method semantics.

Enumerating all the extra constraints Solid added to method semantics over rfc9110 is beyond the purpose of this document (TODO should provide the gist in future). But following is a *very high level (and incomplete)* gist of the actions, server needs to perform on the server state (ignoring authorization), against requests with each of methods:

* `GET`: Server state won't change.
    * **If target is a represented resource::**
        Must resolve a representation, preferably honouring content-negotiation, range headers, and respond with appropriate response, honouring conditional requests (MUST) and many other requirements specified about metadata.
* `POST`: 
    * **[If target is a represented container](https://solidproject.org/ED/protocol#server-post-container)**: After honoring any conditional headers on the request target, must create a new resource as a child of the target container, with server's chosen uri (preferably honouring `Slug` header), with resource kind (container/non-container) determined from `Link` header with `type` rel, and with representation enclosed in the request body.
* `PUT/PATCH`:
    * **If target resource exists**: After honoring any conditional headers on the request target, must update the representation of target resource with representation resolved from the body (after resolving against any patch).
    * **If target resource doesn't exists**: After honoring any conditional headers on the request target, applying mandatory uri semantics, must create any non existing intermediate containers, and finally the target resource with the representation resolved from the body (after resolving against any patch).
* `DELETE`:
    * **If target resource exists**: After honoring any conditional headers on the request target, must delete the target resource and it's auxiliary resources, when resource is not a non empty container. if it is a non-empty container, must return error.
  
Through out all the interactions, the server is expected to uphold the resource space invariants discussed in previous section, without any corruption.

> ⚠️ Though Manas won't explicitly assume uri slash semantics, it provides a more general  model to which a custom uri policy can be plugged in. That way, though the core engine and architecture is agnostic of any perticular semantics over uris, it is easy to enforce semantics induced requirements.

## Decomposing the interaction model

The interaction model of the solid server is not simple. For each request, status of many resource may have to be resolved. Each request can affect state and status of many resources other than the request target. Effects also propagate through resource hierarchies. The problem gets much complicated, if authorization is introduced. Authorization even may have to understand semantics of the patch representations. And all the invariants of the storage layout, and uri slash semantics must be guarded.

The list of requirements together doesn't provide required comprehensibility of all interactions going on. This makes it complicated to architect a server with required functional and quality properties.

Existing open implementations unfortunately doesn't unwind the mud ball from this level. Current state of the art implementations like CSS requires to implement deep abstractions at this level. The provided default implementations of the required deep abstractions are [not concurrent safe](https://github.com/CommunitySolidServer/CommunitySolidServer/issues/1636), and their induced system properties (like ACID properties) [are](https://github.com/CommunitySolidServer/CommunitySolidServer/issues/1429) [not clear](https://github.com/CommunitySolidServer/CommunitySolidServer/issues/1584), highly [coupled](https://github.com/CommunitySolidServer/CommunitySolidServer/issues/998), and are highly imperative and difficult to comprehend.

Thus, it is required to freshly work on decomposing the interactions, and come at clear and flexible abstractions, which can allow for required functional and quality properties for the system.

Following are some of the required properties of the system:

* Functional:
    * Satisfies protocol requirements.
    * Provable concurrency safety over resources and the storage layout
    * Clear ACID semantics over resources and layout.
    * Performance
* Qualitative:
    * Comprehensibility
    * Flexibility
    * Reusability
    * Modularity

### The abstraction of `Resource Operation`

When the solid's interaction model and authorization systems are analyzed, a fundamental unit of `Resource Operation` can be clearly extracted. Method actions are composed of many operations on many resources. Access control is applied against resource operations.

It's semantics have to be fine tuned in light of protocol requirements.

Then it will be possible to have one entity to carry out these resource operations, and higher method services to invoke them in concurrent safe way.

### The abstraction of `Repo`

`Repo` is one of the central abstraction trait in Manas. An instance of `Repo` implementation manages resources in a *single* storage space. It provides services to carry out `Resource Operation`s on the resources it manages.

> [`manas_repo`] crate defines trait definitions for repo, repo context and repo services.

A repo must encapsulate all it's context in a single context object which will be arced.

Each repo implementation associates resource operator services that handle crud resource operations against resources in the storage space it manages. Getting the correct interface to these operator services was critical.

The design of repo trait and it's operator service interfaces is one of the primary contributions, that Manas makes to solid ecosystem.

`Repo` requires an associated `Status token resolver` service, that resolves a `status token` for each resource. Each status token is a sum type, that can be in any of the variants of `Existing + Represented`, `Existing + Non represented`, `Non-existing + Conflict`, `Non-existing + no-conflict`. A caller must first resolve the resource token for any of the resource it want to *touch*. These tokens are opaque to the callers. They only provide minimal interface to resource metadata as per variant. Each `Repo` implementation can *stuff in* what ever internal state into these tokens. These tokens serves as reified proofs for the status of the resource.

Then when caller want to call repo's resource operator services for `CRUD` operations, their `signature` itself demands proper variants of resource tokens to be passed in. For example, `ResourceReader` operator  requires to pass in a token of `Existing + Represented` variant. And `ResourceCreator` operator require to pass in two tokens. One for the resource to be created with variant `NonExisting + ConflictFree`, and one for it's parent container with variant `Existing + Represented`.

This design eliminates all most all of the friction in modularizing the interaction model. For:

1. It delegates the responsibility of ensuring valid state of all resources to callers. Thus callers must first acquire tokens for all the resource they need touch, and pattern match them to ensure correct variant required for the operation, and submit that token variant as proof to resource operators.
2. The concurrency model becomes crystal clear. Caller must follow the thumb rule: "Acquisition and dropping of a resource token must be wrapped in an appropriate lock over the resource name". Thus it is possible to compose this principle, and implement two-phase lock strategy for all compound operations at the caller.
3. Higher callers can use status and minimal metadata provided by the token, to take status dependent decisions efficiently without extra calls, and worrying separately about concurrency.
4. `Repo` implementations can stuff in intermediate state in the token. So that the resource operators can continue from that state, without duplicating the io calls or computation.
5. It becomes trivial to correctly implement layered repos.

Indeed, the design of `Repo` trait, it's resource operator services is a cornerstone of the Manas's architecture.

### Opendal Repo

[`manas_repo_opendal`] crate provides the default implementation of `Repo`, over the object store abstraction layer provided by the [`OpenDAL`].

It allows to support fs, s3, gcs etc backends in robust way.

The repo implementation is tuned to extract maximum value of the backend. For example, while it makes n + 1 number of calls to fs backend for contained resource metadata, it makes only one call to s3 backend by using it's list metadata.

It provides ACID guarantees on resource state and storage layout, when used with flat object stores.

It provides strong validators, when the backend supports.

It is possible to configure and instantiate backends with all advanced configurations thanks to OpenDAL.

### Repo layers

The design of [`Repo`]allows for hassle free reusability of functionality. Thus a basic repo implementation can be layered with many levels of functionality.

[`manas_repo_layers`] crate provides few of them. Like following:

* [`manas_repo_layers::dconneging::DerivedContentNegotiatingRepo`]: A repo, that layers conneg over an inner repo. It is generic over conneg strategy. Many common strategies are provided out of the box.
* [`manas_repo_layers::patching::PatchingRepo]: A repo, that layers patching support over inner repo. It is generic over patcher. N3 solid-insert patcher is provided by default.
* [`manas_repo_layers::validating::ValidatingRepo`]: A repo, that layers validation support over inner repo. This provides extensive and efficient framework for customized validations of resource state change. Out of the box, validators for container representation, aux resource protection, and a multiplexing validator are provided.
* [`manas_access_control::layered_repo::AccessControlledRepo`]: A repo, that layers access control over inner repo. It is generic over a policy enforcement point. Out of the box, support for WAC, and ACP is included.

New layers will be constantly added behind compilation feature gates, as new common functionalities emerge.

Following is a snippet from an assembled recipe, that can demonstrate the flexibility of the architecture.

```rust
/// Type of base opendal repo for the recipe.
pub type RcpBaseRepo<Backend> = OpendalRepo<RcpBaseRepoSetup<Backend>>;

/// Type of conneg layered repo for the recipe.
pub type RcpConnegingRepo<Backend, CNL> = DerivedContentNegotiatingRepo<RcpBaseRepo<Backend>, CNL>;

/// Type of rep validator for the recipe.
pub type RcpRepValidator<Backend, CNL> = MultiRepUpdateValidator<
    RcpConnegingRepo<Backend, CNL>,
    HList!(
        // Validation layer that ensures validity of container reps.
        ContainerProtectingRepUpdateValidator<RcpConnegingRepo<Backend, CNL>, BinaryRepresentation>,
        // Validation layer that ensures validity of aux resource reps.
        AuxProtectingRepUpdateValidator<RcpConnegingRepo<Backend, CNL>, BinaryRepresentation>,
    ),
>;

/// Type of representation patcher for the recipe.
pub type RcpRepPatcher = BinaryRdfDocPatcher<
    RcpStorageSpace,
    // N3 solid-insert-delete patcher.
    SolidInsertDeletePatcher<RcpStorageSpace, HashSet<ArcQuad>>,
    HashSet<ArcQuad>,
>;

/// Type of the repo for the recipe.
/// Recipe uses access-control, rep-patching, rep-validating, and
/// conneg layered opendal repo as it's repo.
pub type RcpRepo<Backend, CNL, PEP> = AccessControlledRepo<
    PatchingRepo<
        ValidatingRepo<RcpConnegingRepo<Backend, CNL>, RcpRepValidator<Backend, CNL>>,
        RcpRepPatcher,
    >,
    PEP,
>;
```

### Storage and StorageService:

[`manas_storage`] crate defines the traits, and implementations of `Storage`, and `StorageService`.

On top of the `Repo` abstraction is `Storage`. A `Storage` is the trait to hold all the specification of a storage. Including it's storage space, context of it's repo, it's method policy, etc.

A `StorageService` is the trait for http service over a storage. Default implementation is provided, that routes requests to method specific services, that handle all the complexities of Solid interaction model in concurrent-safe and clean way. These can be further customized with custom marshallers, or custom layers etc.

### The Podverse

[`manas_podverse`] models extensive abstractions like `Pod`, `PodSet`, `PodService`, `PodSetService`, `PodVerse`, etc. These all together enable for flexible management of pod verse. Many default implementations are provided.

Custom podset implementations, that expose solid pod management as Solid resources is a work-in-progress. It will enable to manage pod provisions as access controllable solid resources.

### Authentication

[`manas_authentication`] crate provides extensive abstractions for challenge-response framework as defined in rfc9110.

By default, it provides complete pluggable support for Solid-OIDC. It can be used in contexts outside of Solid servers too.

Support for `HttpSig` scheme a work-in-progress.

Along with this crate, Manas project provides following crates that deal with identity, and authentication.

* [`webid`]: A crate for representing, dereferencing webids, that integrates into Manas ecosystem. It's functionality will be extended to support profiles, etc.
* [`solid-oidc-types`]: A crate that provides common types for Solid oidc. This can be used in IDP,RS, RP, etc.
* [`dpop`]: A crate for dpop support.

### Access control

[`manas_access_control`] crate provides generic support for resource access control.

It organizes entities involved in access control into following categories inspired by XACML:

* `Policy Enforcement Point (PEP)`: Entity responsible for enforcing the access policies. Out of the box, a trivial PEP that enforce no policy, and a solid compatible PEP are provided.
* `Policy RetrievalPoint (PRP)`: Entity responsible for retrieving the policies of a target resource. Usually a `Repo` implementation also provides an implementation of PRP.
* `Policy Decision Point (PDP)`: Entity responsible for making access decisions based on policies and context. Out of the box `WacDecesionPoint`, and `AcpDecesionPoint` are provided that confirms to `WAC`, and `ACP` specifications respectively. These are highly extensible, with ability to configure custom async matchers, and custom policies on aux resources.

Along with these, crate also provides a layered repo implementation, that layers an inner repo with comprehensive access control.

Manas project provides following related crates:

* `acp`: Crate models the domain of ACP. It also provides extensible evaluation engine.


## Wiring up the recipe

With all the above abstractions, it is required to assemble the final recipe. Currently, there is no custom DI framework being used. Instead it is highly recommended to have a binary crate that can assemble the recipe from clean state in type safe rust.

[`manas_server`] crate assembles recipes for default distribution. It is advised to use that as a reference template for custom recipes.

