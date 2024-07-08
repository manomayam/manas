# Annexe I: Project plan Manas

Manas project aims to make Solid ubiquitous by creating an ecosystem
with well-tested, standards compliant, reusable components in rust and
js, with which one can assemble customized, feature rich Solid storage
servers, clients, and applications, and digital-commons with
data-sovereignty, collaboration, interoperability at the core.

Till now, the project has already handcrafted the required macro
architecture to enable the stated goals. The architecture is documented
in detail at the Manas architecture document
(https://manomayam.github.io/manas/architecture.html). Along with that,
many of the essential components to assemble custom servers were also
delivered. They include but not limited to authentication with
Solid-OIDC, access control with WAC, ACP, repository implementations
backed by fs, major cloud object stores, Solid compliant storage
service, interfaces for pod management, repo layers to add custom
validation, content-negotiation, patching, etc. Few general crates to
help in rdf& solid-ecosystem are also published. Few pre assembled
server recipes also were shipped.

With that work as a solid foundation, the next phase of the project aims
to solidify, and deliver the remaining ecosystem to achieve stated
goals. The tasks for this stage can be broadly classified into following
categories: \> Testing + docs, \> Server components, \> Client
components, \> Application meta components, \> UI components, \>
Devops + Cloud infra.

## 1. Develop the test suite

Existing codebase already exceeds 75k lines of code. But the test
coverage is very less. This phase thus creates an extensive, reusable
test suite to test the abstractions and components. Test suite will be
classified into two categories:

1. Blackbox tests around each layer of abstractions. These will test
    around the architectural abstractions instead of implementation
    specific invariants. This test suite will be reusable. That way, any
    custom implementations can be tested directly without their custom
    tests again.
2.  Implementation specific invariant testing for shipped components.
    These will test the implementation invariants of the default
    components Manas ships.

This two edged testing can make sure that the modularity of the project
doesn’t sacrifice correctness and reliability. Also enables reuse of the
test suite by custom implementations.

### Milestone(s)

1.  Blackbox testsuite for Repo abstraction, implementation invariant
    tests for OpendalRepo, testing detailed invariants of object store
    layout against fs, s3, gcs, backends
2.  Tests for provided repo layers
3.  Blackbox testsuite for SolidStorageService abstraction,
    implementation invariant testing for the default implementation,
    including resource layout, concurrency, ACID semantics, etc.
4.  Test suite for WAC, ACP access control systems.
5.  Test suite for Solid-OIDC resource-server component
6.  Integrate solid-contrib’s conformance-test-harness,
    web-access-control-tests, solid-crud-tests into CI/CD.


## 2. Create high level documentation

Though API is highly documented, and code is well commented, there is a
lack of high level documentation for both end users and custom
implementors. This task will address the concern.

### Milestone(s)

1.  High level documentation for end users
2.  High level documentation for custom implementors and integrators.
3.  Create a logo for the project

## 3. Modules for podverse management

Currently, interfaces and architecture for managing multi-pod podsets,
including provisioning, routing, etc are already created. But concrete
components are yet to follow for dynamic podsets.

### Milestone(s)

1.  Evolve an ontology for pod management, with consideration of state
    of the art.
2.  Implement dynamic pod provisioning as a side-effect of creation of a
    provision resource. It will enable managing pods through solid’s
    resource api itself, and allows to reuse existing access control
    mechanism
3.  Create an admin user interface to manage the pods.

## 4. Cloud ops and cloud native modules

This task focuses on making Manas servers easy to integrate into cloud
stacks with required configurations.

### Milestone(s)

1.  Develop containerization and orchestration primitives for Manas
    servers.
2.  Components to allow for Highly-Available configurations. These
    include but not limited to locker implementations backed by
    redis/dynamodb, event store components backed by
    rabbitmq/aws-event-queue, etc.

## 5. Modules for notifications protocol

Solid notifications protocol allows for interoperable notifications on
the state of solid resources. This task will create required modules for
both servers and clients to efficiently integrate notifications
functionality.

### Milestone(s)

1.  Create a shared crate modeling the world of notifications protocol.

2.  Create a pluggable notifying-repo-layer. This will allow any custom
    server to layer notification seeding functionality over their chosen
    repo implementation.
3.  Implement webhook-channel-2023 module for server and clients.
4.  Implement websocket-channel-2023, eventsource-channel-2023 modules
    for server and clients.

## 6. Specify and implement WebId-SASL authentication scheme

Solid-OIDC is convenient for delegated access by third parties. But is
not suitable for one-to-one authentication with pods. Thus creation of
server side bots etc. is not handled properly in the current ecosystem.
By integrating into SASL ecosytem, we can provide a pluggable
architecture for WebId authentication. This task aims to specify
WebId-SASL, and provide a reusable module to integrate as an auth
scheme. It will also require collaboration with CG (In which the author
is a long time participant).

### Milestone(s)

1.  Specify WebId-SASL.
2.  Create a server module to integrate WebId-SASL auth scheme.
3.  Implement SASL mechanisms specified by \<RFC3163: ISO/IEC 9798-3
    Authentication SASL Mechanism\> as exemplary WebId-SASL mechanisms


## 7. Auxiliary service modules for server

This task involves creating a few auxiliary service modules, that enable
serious usage of data in solid pods in many important domains.

### Milestone(s)

1.  Implement an indexing module. It will index the linked data in a pod
    according to user specified configuration. Also provide index
    interface through quad-pattern-fragments, triple-pattern-fragments,
    and sparql.
2.  Create iiif-image and iiif-av module. Allows for efficient
    consumption of multimedia stored in pods through industry standard
    apis. Highly useful for galleries/libraries/archives/museums.
3.  Module for Memento compatible resource versioning and history.

## 8. Client modules

This task creates essential modules for rust clients against Solid
servers.

### Milestone(s)

1.  Authentication client with solid-oidc, WebId-SASL support.
2.  High level client to easily deal with data in pods, notifications,
    access control, etc.
3.  Provide python, js, wasm bindings to above clients.

## 9. Native application modules

For the applications, a reusable crate will be created to package them
as native applications using Tauri, and Manas. This could make Solid an
attractive storage API to code web & native apps with a single code
base. Already a POC solution was created at Solvent project, that
demonstrates the capabilities of this combination.

### Milestone(s)

1.  Crate to package solid web apps as native apps against the user’s
    filesystem.
2.  Add support for native first applications, that can work on fs, and
    sync back to solid pods.

## 10. UI components

Linked data ecosystem currently lacks even fundamental pieces of
reusable, end-user friendly, accessible, localizable UI components for
basic needs. This creates a formidable barrier in linked data adoption.
This broad task is thus focused on creating those fundamental
components.

To enable reusability, components will be coded as standard web
components, that can be used with any framework, or vanilla html.

To ensure accessibility, and themability, they will be coded on top of
Shoeace web components.

To ensure their fitness for various use case scenarios, they will be
coded as loosely coupled, configurable components.

### Milestone(s)

1.  Component for user authentication with their WebId
2.  Component for ontology+data driven RDF authoring. This aims to
    provide a fluid experience, by making authoring of linked data
    statements as simple as authoring natural language statements.
3.  Component for visualizing, navigating, searching RDF graphs.
4.  Component for access control management.
5.  Component for RDF data gleaning.

## 11. Integrate rauthy identity provider

Rauthy (https://github.com/sebadob/rauthy) is an identity provider
written in rust. This author had worked with the project to add support
for Solid-OIDC to the idp component. This task aims to further make
rauthy pluggable, and create an integrated server that serves both idp,
and solid pods. This will enable quick dev setups for application
developers, and also personal pod+idp hosting.

### Milestone(s)

1.  Make rauthy pluggable, and create a server component to easily
    integrate the idp.

## 12. Processing feedback from Accessibility and Security audits

Project involves developing ui components, and secure Authn/z systems,
apis etc.Hence audits are greatly appreciated, and processing their
feedback will be this task's aim.

### Milestone(s)

1.  Release with feedback processed

[1] NGI Zero Entrust was established with financial support from the
European Commission's Next Generation Internet programme, under the
aegis of DG Communications Networks, Content and Technology.

[2] IEEE Code of Ethics, see: https://www.ieee.org/about/ethics
