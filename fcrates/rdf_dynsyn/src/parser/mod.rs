//! I define dynsyn rdf parsing utilities.
//!

use std::io::BufRead;

use gdp_rs::predicate::impl_::all_of::IntoPL;
use sophia_api::{
    prelude::{Iri, MutableDataset, QuadParser, TripleParser},
    source::{StreamError, TripleSource},
};

use self::{
    error::DynSynParseError, quads::DynSynQuadParserFactory, triples::DynSynTripleParserFactory,
};
use crate::syntax::{
    invariant::parsable::DynSynParsableSyntax,
    predicate::{IsDatasetEncoding, IsGraphEncoding},
};

pub mod error;
pub mod quads;
pub mod triples;

/// A struct for set of rdf parsing factories.
#[derive(Debug, Clone, Default)]
pub struct DynSynParserFactorySet {
    /// Quads parsing parser factory.
    pub quads_parsing: DynSynQuadParserFactory,

    /// Tripes parsing parser factory.
    pub triples_parsing: DynSynTripleParserFactory,
}

impl DynSynParserFactorySet {
    /// Parse quads from given reader,and collect into
    /// provided dataset..
    /// If syntax is triples representing syntax, then
    /// default graph will be the graph name of parsed quads.
    pub fn parse_collect_quads<R, D>(
        &self,
        data: R,
        base_iri: Option<Iri<String>>,
        syntax: DynSynParsableSyntax,
        sync: &mut D,
    ) -> Result<usize, StreamError<DynSynParseError, D::MutationError>>
    where
        R: BufRead,
        D: MutableDataset,
    {
        // convert predicate into predicate-list.
        let syntax = syntax.infer::<IntoPL<_, _>>(Default::default());

        if let Ok(qp_syntax) = syntax.try_extend_predicate::<IsDatasetEncoding>() {
            sync.insert_all(
                self.quads_parsing
                    .new_parser(qp_syntax, base_iri)
                    .parse(data),
            )
        }
        // Else If syntax is triples parsable
        else if let Ok(tp_syntax) = syntax.try_extend_predicate::<IsGraphEncoding>() {
            sync.insert_all(
                self.triples_parsing
                    .new_parser(tp_syntax, base_iri)
                    .parse(data)
                    .to_quads(),
            )
        } else {
            // If syntax is parsable, it must be either
            // quads parsable or triples parsable as of now.
            unreachable!()
        }
    }
}

#[cfg(feature = "async")]
mod async_ {

    use bytes::Bytes;
    use futures::{stream::BoxStream, AsyncRead, TryStream, TryStreamExt};
    use gdp_rs::predicate::impl_::all_of::IntoPL;
    use sophia_api::{
        prelude::Iri,
        quad::Spog,
        term::{FromTerm, Term},
    };

    use super::{error::DynSynParseError, DynSynParserFactorySet};
    use crate::{
        syntax::{
            invariant::parsable::DynSynParsableSyntax,
            predicate::{IsDatasetEncoding, IsGraphEncoding},
        },
        util::stream::bytes_stream_to_async_reader,
    };

    impl DynSynParserFactorySet {
        /// Parse quads from given bytes stream, in given parsable syntax.
        /// If syntax is triples representing syntax, then
        /// default graph will be the graph name of parsed quads.
        pub async fn parse_quads_from_bytes_stream<S, T>(
            &self,
            data: S,
            base_iri: Option<Iri<String>>,
            syntax: DynSynParsableSyntax,
        ) -> BoxStream<'static, Result<Spog<T>, DynSynParseError>>
        where
            S: TryStream<Ok = Bytes> + Send + 'static + Unpin,
            S::Error: 'static + Into<Box<dyn std::error::Error + Send + Sync>>,
            T: Term + FromTerm + Send + 'static,
        {
            self.parse_quads_async(bytes_stream_to_async_reader(data), base_iri, syntax)
                .await
        }

        /// Parse quads from given bytes stream, in given parsable syntax.
        /// If syntax is triples representing syntax, then
        /// default graph will be the graph component of parsed quads.
        pub async fn parse_quads_async<R, T>(
            &self,
            data: R,
            base_iri: Option<Iri<String>>,
            syntax: DynSynParsableSyntax,
        ) -> BoxStream<'static, Result<Spog<T>, DynSynParseError>>
        where
            R: AsyncRead + Send + 'static + Unpin,
            T: Term + FromTerm + Send + 'static,
        {
            // convert predicate into predicate-list.
            let syntax = syntax.infer::<IntoPL<_, _>>(Default::default());

            // If syntax is quads parsable
            if let Ok(qp_syntax) = syntax.try_extend_predicate::<IsDatasetEncoding>() {
                self.quads_parsing
                    .new_async_parser(qp_syntax, base_iri)
                    .parse::<T, R>(data)
                    .await
            }
            // Else If syntax is triples parsable
            else if let Ok(tp_syntax) = syntax.try_extend_predicate::<IsGraphEncoding>() {
                Box::pin(
                    self.triples_parsing
                        .new_async_parser(tp_syntax, base_iri)
                        .parse::<T, R>(data)
                        .await
                        .map_ok(|t| (t, None)),
                )
            } else {
                // If syntax is parsable, it must be either
                // quads parsable or triples parsable as of now.
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod test_data {
    //! these data snippets are copied from sophia tests

    use once_cell::sync::Lazy;
    use sophia_api::prelude::Iri;

    pub static DATASET_STR_N_QUADS: &str = r#"
        <http://localhost/ex#me> <http://example.org/ns/knows> _:b1.
        _:b1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/ns/Person> <tag:g1>.
        _:b1 <http://example.org/ns/name> "Alice" <tag:g1>.
    "#;

    pub static DATASET_STR_TRIG: &str = r#"
        @prefix : <http://example.org/ns/> .
        <#g1> {
            <#me> :knows _:alice.
        }
        <#g2> {
            _:alice a :Person ; :name "Alice".
        }
    "#;

    pub static GRAPH_STR_TURTLE: &str = r#"
        @prefix : <http://example.org/ns/> .
        <#me> :knows [ a :Person ; :name "Alice" ].
    "#;

    pub static GRAPH_STR_N_TRIPLES: &str = r#"
        <http://localhost/ex#me> <http://example.org/ns/knows> _:b1.
        _:b1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/ns/Person>.
        _:b1 <http://example.org/ns/name> "Alice".
    "#;

    pub static GRAPH_STR_RDF_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                xmlns="http://example.org/ns/">
        <rdf:Description rdf:about="http://localhost/ex#me">
        <knows>
            <Person>
            <name>Alice</name>
            </Person>
        </knows>
        </rdf:Description>
    </rdf:RDF>
    "#;

    pub static BASE_IRI1: Lazy<Iri<String>> =
        Lazy::new(|| Iri::new("http://localhost/ex".to_owned()).unwrap());
    // pub static G1_IRI: &'static str = "http://localhost/ex#g1";
    // pub static G2_IRI: &'static str = "http://localhost/ex#g2";
}
