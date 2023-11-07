//! I define dynsyn rdf serializing utilities.
//!

use std::io::{self, Write};

use gdp_rs::predicate::impl_::all_of::IntoPL;
use sophia_api::{
    quad::Quad,
    serializer::{QuadSerializer, TripleSerializer},
    source::{QuadSource, StreamResult},
};

use self::{quads::DynSynQuadSerializerFactory, triples::DynSynTripleSerializerFactory};
use crate::syntax::{
    invariant::serializable::DynSynSerializableSyntax,
    predicate::{IsDatasetEncoding, IsGraphEncoding},
};

pub mod config;
pub mod quads;
pub mod triples;

/// A struct for set of rdf serializing factories.
#[derive(Debug, Default)]
pub struct DynSynSerializerFactorySet {
    /// Quads serializing serializer factory.
    pub quads_serializing: DynSynQuadSerializerFactory,

    /// Tripes serializing serializer factory.
    pub triples_serializing: DynSynTripleSerializerFactory,
}

impl DynSynSerializerFactorySet {
    /// Wrapping serialize the given quad source.
    ///
    /// If syntax supports representing quads, it serializes all quads.
    /// Otherwise it serializes triples of default graph.
    ///
    pub fn wrapping_serialize_quads<QS, W>(
        &self,
        quads: QS,
        write: W,
        syntax: DynSynSerializableSyntax,
    ) -> StreamResult<(), QS::Error, io::Error>
    where
        QS: QuadSource,
        W: Write,
    {
        let syntax = syntax.infer::<IntoPL<_, _>>(Default::default());

        // If syntax is quads serializable
        if let Ok(qs_syntax) = syntax.try_extend_predicate::<IsDatasetEncoding>() {
            self.quads_serializing
                .new_serializer(qs_syntax, write)
                .serialize_quads(quads)?;
            Ok(())
        }
        // Else if syntax is triples serializable
        else if let Ok(ts_syntax) = syntax.try_extend_predicate::<IsGraphEncoding>() {
            self.triples_serializing
                .new_serializer(ts_syntax, write)
                .serialize_triples(quads.filter_quads(|q| q.g().is_none()).to_triples())?;
            Ok(())
        } else {
            // A serializable syntax must be either quad serializable or triples serializable.
            unreachable!()
        }
    }
}

#[cfg(feature = "async")]
mod async_ {
    use std::io;

    use futures::AsyncWrite;
    use gdp_rs::predicate::impl_::all_of::IntoPL;
    use sophia_api::{dataset::Dataset, source::StreamResult};

    use super::DynSynSerializerFactorySet;
    use crate::syntax::{
        invariant::serializable::DynSynSerializableSyntax,
        predicate::{IsDatasetEncoding, IsGraphEncoding},
    };

    impl DynSynSerializerFactorySet {
        /// Serialize given dataset to specified syntax asynchronously.
        /// If syntax doesn't support quads, serialize only wrapped default graph.
        // TODO refactor duplication.
        pub async fn wrapping_serialize_dataset_async<D, W>(
            &self,
            dataset: D,
            write: W,
            syntax: DynSynSerializableSyntax,
        ) -> StreamResult<(), D::Error, io::Error>
        where
            D: Dataset + Send + Sync + 'static + Unpin,
            D::Error: Send + Sync + 'static,
            W: Send + 'static + AsyncWrite + Unpin,
        {
            // convert predicate into predicate-list.
            let syntax = syntax.infer::<IntoPL<_, _>>(Default::default());

            // If syntax is quads serializable
            if let Ok(qs_syntax) = syntax.try_extend_predicate::<IsDatasetEncoding>() {
                self.quads_serializing
                    .new_async_serializer(qs_syntax, write)
                    .serialize_dataset(dataset)
                    .await?;
                Ok(())
            }
            // Else if syntax is triples serializable
            else if let Ok(ts_syntax) = syntax.try_extend_predicate::<IsGraphEncoding>() {
                self.triples_serializing
                    .new_async_serializer(ts_syntax, write)
                    .wrapping_serialize_dataset(dataset)
                    .await?;
                Ok(())
            } else {
                // A serializable syntax must be either quad serializable or triples serializable.
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod test_data {
    //! These test data snippets are copied from sophia tests
    //!

    pub static TESTS_NQUADS: &[&str] = &[
        r#"<http://champin.net/#pa> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.
<http://champin.net/#pa> <http://schema.org/name> "Pierre-Antoine" <http://champin.net/>.
"#,
    ];

    pub static TESTS_NTRIPLES: &[&str] = &[
        r#"<http://champin.net/#pa> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.
<http://champin.net/#pa> <http://schema.org/name> "Pierre-Antoine".
"#,
    ];

    pub static TESTS_RDF_XML: &[&str] = &[r#"<?xml version="1.0" encoding="utf-8"?>
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
    "#];

    pub static TESTS_TRIG: &[&str] = &[
        "#empty trig",
        r#"# simple quads
            PREFIX : <http://example.org/ns/>
            :alice a :Person; :name "Alice"; :age 42.
            GRAPH :g {
                :bob a :Person, :Man; :nick "bob"@fr, "bobby"@en; :admin true.
            }
        "#,
        r#"# lists
            GRAPH <tag:g> { <tag:alice> <tag:likes> ( 1 2 ( 3 4 ) 5 6 ), ("a" "b"). }
        "#,
        r#"# subject lists
            GRAPH <tag:g> { (1 2 3) a <tag:List>. }
        "#,
        r#"# blank node graph name
            PREFIX : <http://example.org/ns/>
            #:lois :belives _:b.
            #GRAPH _:b1 { :clark a :Human }
        "#,
        r#"# list split over different graphs
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            _:a rdf:first 42; rdf:rest _:b.
            GRAPH [] {
                _:b rdf:first 43; rdf:rest ().
            }
        "#,
    ];

    pub static TESTS_TURTLE: &[&str] = &[
        "#empty ttl",
        r#"# simple triple
            PREFIX : <http://example.org/ns/>
            :alice a :Person; :name "Alice"; :age 42.
            :bob a :Person, :Man; :nick "bob"@fr, "bobby"@en; :admin true.
        "#,
        r#"# lists
            <tag:alice> <tag:likes> ( 1 2 ( 3 4 ) 5 6 ), ("a" "b").
        "#,
        r#"# subject lists
            (1 2 3) a <tag:List>.
        "#,
        r#"# malformed list
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            _:a rdf:first 42, 43; rdf:rest (44 45).
            _:b rdf:first 42; rdf:rest (43), (44).
        "#,
        r#"# bnode cycles
        PREFIX : <http://example.org/ns/>
        _:a :n "a"; :p [ :q [ :r _:a ]].
        _:b :n "b"; :s [ :s _:b ].
        "#,
    ];
}
