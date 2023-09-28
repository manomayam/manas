# Introduction

[Solid](https://solidproject.org/) is a web native protocol to enable interoperable, read-write, collaborative, and decentralized web, truer to web's original vision. It embraces advancements that happened after web's original conception in domains of data-modelling, identity, access control, trust, etc, and standardizes notions for few classes of entities, and interactions between them. It is thin, and closer to web.

Manas project aims to create a modular framework and ecosystem to create correct, robust storage servers adhering to [Solid protocol](https://solidproject.org/TR/protocol) in rust.

Manas aims to make solid ubiquitous. Thus it wants the server to be so easy to deploy, and can run on low-resource raspberries to low-latency serverless. From app developer machines to their ci-cd setups. And as also as a base for native applications. It must be also easy to assemble custom servers with what ever backends, extensions, layers.

To achieve that, Manas's architecture is crafted keeping following desired properties of the system in mind.

* Correct,seamless, and reliable.
* Modularity, reusability, extendability.
* Well defined performance characteristics.
* Well defined ACID properties.
* Well defined concurrency.

For that, one of the choice made is to use rust as the primary language.
* It allows produce single binary servers that can be deployed easily without hassle of runtimes.
* Has low resource usage, and low startup times.
* Rust + strict type driven design, enables and guides to encode  invariants at compile time. Allows for robust software.
* Code can be reused as backend for native applications with [Tauri](https://tauri.app/).

Manas thus models robust, definitive abstractions for many aspects of http and solid protocols in modular way, and provides them in well factored crates. This enables shared understanding of the domain, and to assemble customized server recipes.
