# Summary

Manas project aims to make Solid ubiquitous by creating an ecosystem with well-tested, reusable components in rust and js, with which one can assemble customized, feature rich Solid storage servers, clients, and applications, and digital-commons with data-sovereignty collaboration at the core.

Using rust, the servers could be run on low resource raspberry-pies to low latency serverless clouds, or as lightweight developer test servers. Can use custom storages from filesystem, object-stores, or consumer cloud storages like google-drive as backends. Support for WAC, ACP authorization systems, Solid-OIDC, HTTPSig authentication schemes, multi pod management, solid-notifications, etc will be provided as reusable layers. And the layered architecture enables adding customized validation, or any other custom features.

For clients, a rust client, and other helper crates will be developed for Solid protocol, Solid-notifications, etc, with probable bindings to other languages, that enables small CLIs, and other server-side/client side applications.

For the applications, a reusable crate will be created to package them as native applications using tauri, and Manas. This could make Solid an attractive storage api to code web & native apps with a single code base. It can be extended to offer sync solutions, native-first apps, etc in future.


# Work items:

The basic framework with support for creating single pod Solid compatible servers is already there. The architecture is explained in the [book](https://manomayam.github.io/manas/architecture.html).

We can currently use object-stores, fs as backends. Authentication with Solid-OIDC was implemented. Access control with WAC, ACP are provided as layers one can choose. etc.

Beyond that following will be worked upon before hitting 1.0

## Server roadmap

- [ ] Write tests for `manas_repo_opendal` default repo implementation.
- [ ] Integrate solid-contrib's [conformance-test-harness](https://github.com/solid-contrib/conformance-test-harness) in CI/CD #10
- [ ] Migrate to hyper 1.0 and it's new ecosystem.
- [ ] Implement dynamic pod provisioning, with provision resources as Solid resources. (WIP)
- [ ] Create a crate for modelling the world of [Solid Notifications Protocol](https://solid.github.io/notifications/protocol), that can be used by both servers and clients.
- [ ] Implement server side notifications. This will also be used for multi-instance deployments to invalidate caches, etc.
- [ ] Dockerize, and create helm charts for deploying with horizontally scalable configurations.
- [ ] Create recipes that can be launched instantly, backed by serverless stacks of AWS, AZURE. This will allow easy cost-effective, serverless pod deployments.
- [ ] Add support for HttpSig authentication protocol. It will enable agents to authenticate directly, with out idp. Will ease creation of bots
- [ ] Create auxiliary, pluggable services, for indexing, versioning, provenance, iiif, labels, etc.
- [ ] etc.


## Client roadmap

- [ ] Create authentication client with solid-oidc, httpsig support, with wasm32-browser target also being supported.
- [ ] Create rdfjs compatible js bindings to sophia.
- [ ] Create high level client, to work with data in solid servers.
- [ ] Add notifications-protocol support.
- [ ] Provide python, js bindings.

## Applications roadmap.

- [ ] Create a support crate, to easily nativify Solid applications with tauri. This can be iterated first at [Solvent](https://github.com/manomayam/solvent).
- [ ] Create web components for managing WAC/ACP access-controls.
- [ ] Create web components for easy integration of authentication.
- [ ] Create web components for basic resource browsing and selection.
- [ ] etc.
