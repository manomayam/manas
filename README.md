# मनस् | Manas

[Solid](https://solidproject.org/) is a web native protocol to enable interoperable, read-write, collaborative, and decentralized web, truer to web's original vision.

> Solid adds to existing Web standards to realise a space where individuals can maintain their autonomy, control their data and privacy, and choose applications and services to fulfil their needs.

Manas project aims to create a modular framework and ecosystem to create correct, robust storage servers adhering to [Solid protocol](https://solidproject.org/TR/protocol) in rust.

Manas thus models robust, definitive abstractions for many aspects of http and solid protocols in modular way, and provides them in well factored crates. This enables shared understanding of the domain, and to assemble customized server recipes.

> ⚠️ Note that, though much of the feature set is implemented, and architecture is relatively stable, it's test suite is constantly evolving. Many of devops bells are being integrated, and apis are being refined. Thus project is considered to be in alpha stage until it reaches at least version 0.5

## Default server recipes
For end users, Manas provides officially supported assembled servers in [manas_server](./crates/manas_server/) crate. They support following features:

* Support for Fs/S3/GCS and other object stores as backends through [OpenDAL](https://github.com/apache/incubator-opendal) abstraction layer.
* Authentication using Solid-OIDC protocol
* Access control adhering to WAC/ACP.
* PATCH requests with n3-patch support.
* Full support for conditional requests, range requests, content-negotiation.
* Integrated solid-os databrowser frontend.
* ..etc.

## For developers

[![Docs.rs](https://docs.rs/manas/badge.svg)](https://docs.rs/manas)

It is highly recommended to read the [book](https://manomayam.github.io/manas/) to understand Manas's architecture.

## Overview of crates

It provides following crates:

- [`manas_http`](https://docs.rs/manas_http): Provides extended functionality for handling http semantics, that integrates into [`hyper`](https://docs.rs/hyper/latest/hyper/index.html) ecosystem.
    - Provides comprehensive list of typed headers for solid ecosystem.
    - Provides types for various invariants of http uris, (like absolute, normalized, etc.)
    - Defines trait and implementations for http representation.
    - Provides implementation of conditional requests related algorithms.
    - etc.

- [`manas_authentication`](https://docs.rs/manas_authentication): Defines traits for http challenge-response based authentication schemes. Provides default implementations confirming to [`Solid-OIDC`](https://solid.github.io/solid-oidc/).

- [`manas_space`](https://docs.rs/manas_space): Defines traits for abstractions that together define a solid server's resource space confirming to generalized solid model

- [`manas_repo`](https://docs.rs/manas_repo): Defines trait for defining backend repositories and their services. This trait is plugging point for supporting custom backend repositories.

- [`mans_repo_opendal`](https://docs.rs/manas_repo_opendal): Provides default repository implementation on top of [OpenDAL](https://docs.rs/opendal/latest/opendal/) object store abstraction layer.
   Through OpenDAL, it supports backends such as  `fs`, `s3-compatible`, `gcs`, `azblob`, etc. out of the box. While also allowing to take advantage of it's layer interface for enabling retry, tracing, etc.
   Through implementing OpenDAL's [`Accessor`](https://docs.rs/opendal/latest/opendal/trait.Accessor.html), one can plug to this repository implementation for any object-store like backends, instead of reimplementing entire repository interface.

- [`mans_repo_layers`](https://docs.rs/manas_repo_opendal_layers): Provides few layering repo implementations, that can be  layered over any repos to provide functionality like patching, validation, content-negotiation, etc.

- [`manas_access_control`](https://docs.rs/manas_access_control): Defines traits for Access control systems compatible with solid storage space. Provides default implementations confirming to [`ACP`](https://solid.github.io/authorization-panel/acp-specification/), [`WAC`](https://solid.github.io/web-access-control-spec/) authorization systems. Also provides an authorization layering repo implementation, to add authorization over any inner repo.

- [`manas_storage`](https://docs.rs/manas_storage): Provides traits for a `SolidStorage`, and `SolidStorageService`, (a solid-protocol compatible http service over a storage). Also provides default, modular implementations of concurrent safe http method services, confirming to solid protocol.

- [`manas_podverse`](https://docs.rs/manas_podverse): Provides traits (`Pod`, `PodSet`, and `ProvisionablePodSet`, etc.) and default implementations for defining, serving, provisioning solid pods and podsets.

- [`manas_server`](https://docs.rs/manas_server): Provides default recipes of solid server.

- [`manas`](https://docs.rs/manas): All inclusive crate.


Along with these main crates, Manas project provides many utility crates that help dealing with solid ecosystem.

- [`dpop`](https://docs.rs/dpop): Rust crate for dpop protocol
- [`rdf_dynsyn`](https://docs.rs/rdf_dynsyn): Rust crate for dynamic rdf parsers and serializers on to of sophia.
- [`rdf_utils`](https://docs.rs/rdf_utils): Utilities to handle rdf in rust.
- [`webid`](https://docs.rs/webid): Rust crate for webid.
- ... etc.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
