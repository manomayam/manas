//! This bin generates solid protocol spec mod.
//!

use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use manas_specs_codegen::{
    gen_spec_mod::{SpecCodegenConfig, SpecGraph},
    util::graph::GraphExt,
};
use rdf_utils::model::triple::ArcTriple;
use sophia_api::prelude::IriRef;

static PROTOCOL_TTL_STR: &str = include_str!("ed.ttl");

fn gen() -> Result<String, anyhow::Error> {
    let spec_graph = SpecGraph {
        id: IriRef::new_unchecked(Arc::from("https://solidproject.org/ED/protocol")),
        title: "Solid Protocol".into(),
        ns_base: IriRef::new_unchecked(Arc::from("https://solidproject.org/ED/protocol#")),
        graph: HashSet::<ArcTriple>::try_from_turtle_str(PROTOCOL_TTL_STR)
            .with_context(|| "Error in parsing spec graph from turtle file.")?,
    };

    let gen_config = SpecCodegenConfig {
        spec_rid: "SolidProtocol".into(),
        req_rid_prefix: "REQ_".into(),
        req_sub_rid_map: [
            (
                "https://solidproject.org/ED/protocol#Server".into(),
                "SUBJECT_SERVER".to_owned(),
            ),
            (
                "https://solidproject.org/ED/protocol#Client".into(),
                "SUBJECT_CLIENT".to_owned(),
            ),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    spec_graph.gen_spec_mod(&gen_config)
}

fn main() {
    let spec_mod = gen().expect("Error in generating spec mod for solid protocol.");

    println!("{}", spec_mod);
}
