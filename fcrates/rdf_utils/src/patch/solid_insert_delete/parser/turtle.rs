//! Implementation of Turtle and Trig RDF syntax

use std::{collections::HashMap, io::BufRead, str};

use oxiri::Iri;
use rio_api::{
    model::*,
    parser::{QuadsParser, TriplesParser},
};

use super::{error::*, shared::*, triple_allocator::TripleAllocator, utils::*};

/// A [Turtle](https://www.w3.org/TR/turtle/) and [Turtle-star](https://w3c.github.io/rdf-star/cg-spec/#turtle-star) streaming parser.
///
/// It implements the [`TriplesParser`] trait.
///
///
/// Count the number of people using the [`TriplesParser`] API:
/// ```
/// use rio_turtle::{TurtleParser, TurtleError};
/// use rio_api::parser::TriplesParser;
/// use rio_api::model::NamedNode;
///
/// let file = b"@prefix schema: <http://schema.org/> .
/// <http://example.com/foo> a schema:Person ;
///     schema:name  \"Foo\" .
/// <http://example.com/bar> a schema:Person ;
///     schema:name  \"Bar\" .";
///
/// let rdf_type = NamedNode { iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" };
/// let schema_person = NamedNode { iri: "http://schema.org/Person" };
/// let mut count = 0;
/// TurtleParser::new(file.as_ref(), None).parse_all(&mut |t| {
///     if t.predicate == rdf_type && t.object == schema_person.into() {
///         count += 1;
///     }
///     Ok(()) as Result<(), TurtleError>
/// })?;
/// assert_eq!(2, count);
/// # Result::<_,rio_turtle::TurtleError>::Ok(())
/// ```
pub struct TurtleParser<R: BufRead> {
    read: LookAheadByteReader<R>,
    base_iri: Option<Iri<String>>,
    prefixes: HashMap<String, String>,
    bnode_id_generator: BlankNodeIdGenerator,
    triple_alloc: TripleAllocator,
    temp_buf: String,
}

impl<R: BufRead> TurtleParser<R> {
    /// Builds the parser from a `BufRead` implementation, and a base IRI for relative IRI resolution.
    pub fn _new(reader: R, base_iri: Option<Iri<String>>) -> Self {
        let mut triple_alloc = TripleAllocator::_new();
        triple_alloc.push_triple_start();
        Self {
            read: LookAheadByteReader::new(reader),
            base_iri,
            prefixes: HashMap::default(),
            bnode_id_generator: BlankNodeIdGenerator::default(),
            triple_alloc,
            temp_buf: String::default(),
        }
    }

    /// The list of IRI prefixes considered at the current step of the parsing.
    ///
    /// This method returns the mapping from prefix name to prefix value.
    /// It is empty at the beginning of the parsing and gets updated when prefixes are encountered.
    /// It should be full at the end of the parsing (but if a prefix is overridden, only the latest version will be returned).
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use rio_api::model::NamedNode;
    /// use rio_api::parser::TriplesParser;
    /// use rio_turtle::{TurtleError, TurtleParser};
    ///
    /// let file = b"@prefix schema: <http://schema.org/> .
    /// @prefix ex: <http://example.com/> .
    /// ex: a schema:WebSite .
    /// @prefix ex: <http://example.org/> .
    /// ex: a schema:WebSite .";
    ///
    /// let mut parser = TurtleParser::new(file.as_ref(), None);
    /// // assert_eq!(parser._prefixes(), &HashMap::new()); // No prefix at the beginning
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the first prefix"));
    /// // assert_eq!(parser._prefixes().len(), 1);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the second prefix"));
    /// // assert_eq!(parser._prefixes().len(), 2);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    /// // assert_eq!(parser._prefixes()["ex"], "http://example.com/");
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |t| {
    ///     assert_eq!(t.subject, NamedNode { iri: "http://example.com/" }.into()); // We read the first triple
    ///     Ok(())
    /// });
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the new version of the ex: prefix"));
    /// // assert_eq!(parser._prefixes().len(), 2);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    /// // assert_eq!(parser._prefixes()["ex"], "http://example.org/");  
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |t| {
    ///     assert_eq!(t.subject, NamedNode { iri: "http://example.org/" }.into()); // We read the second triple
    ///    Ok(())
    /// });
    ///
    /// # Result::<_,TurtleError>::Ok(())
    /// ```
    pub fn _prefixes(&self) -> &HashMap<String, String> {
        &self.prefixes
    }
}

impl<R: BufRead> TriplesParser for TurtleParser<R> {
    type Error = TurtleError;

    fn parse_step<E: From<TurtleError>>(
        &mut self,
        on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
    ) -> Result<(), E> {
        parse_statement(self, on_triple)
    }

    fn is_end(&self) -> bool {
        self.read.current().is_none()
    }
}

/// A [TriG](https://www.w3.org/TR/trig/) and [TriG-star](https://w3c.github.io/rdf-star/cg-spec/#trig-star) streaming parser.
///
/// It implements the `QuadsParser` trait.
///
///
/// Count the number of people using the `QuadsParser` API:
/// ```
/// use rio_turtle::{TriGParser, TurtleError};
/// use rio_api::parser::QuadsParser;
/// use rio_api::model::NamedNode;
///
/// let file = b"@prefix schema: <http://schema.org/> .
/// <http://example/> {
///     <http://example.com/foo> a schema:Person ;
///         schema:name  \"Foo\" .
///     <http://example.com/bar> a schema:Person ;
///         schema:name  \"Bar\" .
/// }";
///
/// let rdf_type = NamedNode { iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" };
/// let schema_person = NamedNode { iri: "http://schema.org/Person" };
/// let mut count = 0;
/// TriGParser::new(file.as_ref(), None).parse_all(&mut |t| {
///     if t.predicate == rdf_type && t.object == schema_person.into() {
///         count += 1;
///     }
///     Ok(()) as Result<(), TurtleError>
/// })?;
/// assert_eq!(2, count);
/// # Result::<_, TurtleError>::Ok(())
/// ```
pub struct TriGParser<R: BufRead> {
    inner: TurtleParser<R>,
    graph_name_buf: String,
}

impl<R: BufRead> TriGParser<R> {
    /// Builds the parser from a `BufRead` implementation, and a base IRI for relative IRI resolution.
    pub fn _new(reader: R, base_iri: Option<Iri<String>>) -> Self {
        Self {
            inner: TurtleParser::_new(reader, base_iri),
            graph_name_buf: String::default(),
        }
    }

    /// The list of IRI prefixes considered at the current step of the parsing.
    ///
    /// This method returns the mapping from prefix name to prefix value.
    /// It is empty at the beginning of the parsing and gets updated when prefixes are encountered.
    /// It should be full at the end of the parsing (but if a prefix is overridden, only the latest version will be returned).
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use rio_api::model::NamedNode;
    /// use rio_api::parser::QuadsParser;
    /// use rio_turtle::{TurtleError, TriGParser};
    ///
    /// let file = b"@prefix schema: <http://schema.org/> .
    /// @prefix ex: <http://example.com/> .
    /// ex: a schema:WebSite .
    /// @prefix ex: <http://example.org/> .
    /// ex: a schema:WebSite .";
    ///
    /// let mut parser = TriGParser::new(file.as_ref(), None);
    /// // assert_eq!(parser._prefixes(), &HashMap::new()); // No prefix at the beginning
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the first prefix"));
    /// // assert_eq!(parser._prefixes().len(), 1);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the second prefix"));
    /// // assert_eq!(parser._prefixes().len(), 2);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    /// // assert_eq!(parser._prefixes()["ex"], "http://example.com/");
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |t| {
    ///     assert_eq!(t.subject, NamedNode { iri: "http://example.com/" }.into()); // We read the first triple
    ///     Ok(())
    /// });
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |_| panic!("We read the new version of the ex: prefix"));
    /// // assert_eq!(parser._prefixes().len(), 2);
    /// // assert_eq!(parser._prefixes()["schema"], "http://schema.org/");
    /// // assert_eq!(parser._prefixes()["ex"], "http://example.org/");  
    ///
    /// let _: Result<_, TurtleError> = parser.parse_step(&mut |t| {
    ///     assert_eq!(t.subject, NamedNode { iri: "http://example.org/" }.into()); // We read the second triple
    ///    Ok(())
    /// });
    ///
    /// # Result::<_,TurtleError>::Ok(())
    /// ```
    pub fn _prefixes(&self) -> &HashMap<String, String> {
        &self.inner.prefixes
    }
}

impl<R: BufRead> QuadsParser for TriGParser<R> {
    type Error = TurtleError;

    fn parse_step<E: From<TurtleError>>(
        &mut self,
        on_quad: &mut impl FnMut(Quad<'_>) -> Result<(), E>,
    ) -> Result<(), E> {
        parse_block_or_directive(self, on_quad)
    }

    fn is_end(&self) -> bool {
        self.inner.read.current().is_none()
    }
}

pub(crate) const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub(crate) const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
pub(crate) const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
pub(crate) const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
pub(crate) const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
pub(crate) const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
pub(crate) const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
pub(crate) const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

fn parse_statement<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    skip_whitespace(&mut parser.read)?;

    if parser.read.current().is_none() {
        Ok(())
    } else if parser.read.starts_with(b"@prefix") {
        parse_prefix_id(
            &mut parser.read,
            &mut parser.prefixes,
            &parser.base_iri,
            &mut parser.temp_buf,
        )
        .map_err(E::from)
    } else if parser.read.starts_with(b"@base") {
        parser.base_iri = Some(parse_base(
            &mut parser.read,
            &mut parser.temp_buf,
            &parser.base_iri,
        )?);
        Ok(())
    } else if parser.read.starts_with_ignore_ascii_case(b"BASE") {
        parser.base_iri = Some(parse_sparql_base(
            &mut parser.read,
            &mut parser.temp_buf,
            &parser.base_iri,
        )?);
        Ok(())
    } else if parser.read.starts_with_ignore_ascii_case(b"PREFIX") {
        parse_sparql_prefix(
            &mut parser.read,
            &mut parser.prefixes,
            &parser.base_iri,
            &mut parser.temp_buf,
        )
        .map_err(E::from)
    } else {
        parse_triples(parser, on_triple)?;

        parser.read.check_is_current(b'.')?;
        parser.read.consume()?;
        Ok(())
    }
}

fn parse_block_or_directive<E: From<TurtleError>>(
    parser: &mut TriGParser<impl BufRead>,
    on_quad: &mut impl FnMut(Quad<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [1g] 	trigDoc 	::= 	(directive | block)*
    // [2g] 	block 	::= 	triplesOrGraph | wrappedGraph | triples2 | "GRAPH" labelOrSubject wrappedGraph
    skip_whitespace(&mut parser.inner.read)?;

    if parser.inner.read.current().is_none() {
        Ok(())
    } else if parser.inner.read.starts_with(b"@prefix") {
        parse_prefix_id(
            &mut parser.inner.read,
            &mut parser.inner.prefixes,
            &parser.inner.base_iri,
            &mut parser.inner.temp_buf,
        )?;
        Ok(())
    } else if parser.inner.read.starts_with(b"@base") {
        parser.inner.base_iri = Some(parse_base(
            &mut parser.inner.read,
            &mut parser.inner.temp_buf,
            &parser.inner.base_iri,
        )?);
        Ok(())
    } else if parser.inner.read.starts_with_ignore_ascii_case(b"BASE") {
        parser.inner.base_iri = Some(parse_sparql_base(
            &mut parser.inner.read,
            &mut parser.inner.temp_buf,
            &parser.inner.base_iri,
        )?);
        Ok(())
    } else if parser.inner.read.starts_with_ignore_ascii_case(b"PREFIX") {
        parse_sparql_prefix(
            &mut parser.inner.read,
            &mut parser.inner.prefixes,
            &parser.inner.base_iri,
            &mut parser.inner.temp_buf,
        )?;
        Ok(())
    } else if parser.inner.read.starts_with_ignore_ascii_case(b"GRAPH") {
        parser.inner.read.consume_many("GRAPH".len())?;
        skip_whitespace(&mut parser.inner.read)?;

        let graph_name = parse_label_or_subject(&mut parser.graph_name_buf, &mut parser.inner)?;
        skip_whitespace(&mut parser.inner.read)?;

        parse_wrapped_graph(
            &mut parser.inner,
            &mut on_triple_in_graph(on_quad, Some(graph_name)),
        )?;
        parser.graph_name_buf.clear();
        Ok(())
    } else if parser.inner.read.current() == Some(b'{') {
        parse_wrapped_graph(&mut parser.inner, &mut on_triple_in_graph(on_quad, None))
    } else if parser.inner.read.current() == Some(b'[')
        && !is_followed_by_space_and_closing_bracket(&mut parser.inner.read)?
        || parser.inner.read.current() == Some(b'(')
    {
        parse_triples2(&mut parser.inner, &mut on_triple_in_graph(on_quad, None))
    } else {
        parse_triples_or_graph(parser, on_quad)
    }
}

fn parse_triples_or_graph<E: From<TurtleError>>(
    parser: &mut TriGParser<impl BufRead>,
    on_quad: &mut impl FnMut(Quad<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [3g] 	triplesOrGraph 	::= 	labelOrSubject ( wrappedGraph | predicateObjectList '.' ) | embTriple predicateObjectList '.'

    if parser.inner.read.starts_with(b"<<") {
        parse_embedded_triple(&mut parser.inner)?;
        parser.inner.triple_alloc.push_subject_triple();
        skip_whitespace(&mut parser.inner.read)?;
        parse_predicate_object_list(&mut parser.inner, &mut on_triple_in_graph(on_quad, None))?;
        parser.inner.read.check_is_current(b'.')?;
        parser.inner.read.consume()?;
        parser.inner.triple_alloc.pop_subject();
        return Ok(());
    }

    let TriGParser {
        inner,
        graph_name_buf,
    } = parser;
    let graph_name = parse_label_or_subject(graph_name_buf, inner)?;
    skip_whitespace(&mut inner.read)?;

    if inner.read.current() == Some(b'{') {
        parse_wrapped_graph(
            &mut parser.inner,
            &mut on_triple_in_graph(on_quad, Some(graph_name)),
        )?;
    } else {
        let blank = matches!(graph_name, GraphName::BlankNode(_));
        inner.triple_alloc.try_push_subject(|b| {
            b.push_str(graph_name_buf);
            if blank {
                Ok(Subject::BlankNode(BlankNode { id: b }))
            } else {
                Ok(Subject::NamedNode(NamedNode { iri: b }))
            }
        })?;
        parse_predicate_object_list(&mut parser.inner, &mut on_triple_in_graph(on_quad, None))?;

        parser.inner.read.check_is_current(b'.')?;
        parser.inner.read.consume()?;
        parser.inner.triple_alloc.pop_subject();
    }
    parser.graph_name_buf.clear();
    Ok(())
}

fn parse_triples2<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [4g] 	triples2 	::= 	blankNodePropertyList predicateObjectList? '.' | collection predicateObjectList '.'
    match parser.read.current() {
        Some(b'[') if !is_followed_by_space_and_closing_bracket(&mut parser.read)? => {
            let id = parse_blank_node_property_list(parser, on_triple)?;
            parser.triple_alloc.try_push_subject(|b| {
                b.push_str(id.as_ref());
                Ok(Subject::from(BlankNode { id: b }))
            })?;
            skip_whitespace(&mut parser.read)?;
            if parser.read.current() != Some(b'.') {
                parse_predicate_object_list(parser, on_triple)?;
            }
        }
        _ => {
            let collec = parse_collection(parser, on_triple)?;
            parser
                .triple_alloc
                .try_push_subject(|b| allocate_collection(collec, b))?;
            skip_whitespace(&mut parser.read)?;
            parse_predicate_object_list(parser, on_triple)?;
        }
    }

    parser.triple_alloc.pop_subject();

    parser.read.check_is_current(b'.')?;
    parser.read.consume()?;
    Ok(())
}

fn parse_wrapped_graph<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [5g] 	wrappedGraph 	::= 	'{' triplesBlock? '}'
    // [6g] 	triplesBlock 	::= 	triples ('.' triplesBlock?)?
    parser.read.check_is_current(b'{')?;
    parser.read.consume()?;
    skip_whitespace(&mut parser.read)?;

    loop {
        if parser.read.current() == Some(b'}') {
            parser.read.consume()?;
            break;
        }

        parse_triples(parser, on_triple)?;
        match parser.read.current() {
            Some(b'.') => {
                parser.read.consume()?;
                skip_whitespace(&mut parser.read)?;
            }
            Some(b'}') => {
                parser.read.consume()?;
                break;
            }
            _ => parser.read.unexpected_char_error()?,
        }
    }
    Ok(())
}

fn parse_label_or_subject<'a>(
    buffer: &'a mut String,
    parser: &mut TurtleParser<impl BufRead>,
) -> Result<GraphName<'a>, TurtleError> {
    //[7g] 	labelOrSubject 	::= 	iri | BlankNode
    // (split in two for the case of TriG*)

    let TurtleParser {
        read,
        base_iri,
        prefixes,
        bnode_id_generator,
        temp_buf,
        ..
    } = parser;
    Ok(match read.current() {
        Some(b'_') | Some(b'[') => parse_blank_node(read, buffer, bnode_id_generator)?.into(),
        _ => parse_iri(read, buffer, temp_buf, base_iri, prefixes)?.into(),
    })
}

fn parse_prefix_id(
    read: &mut LookAheadByteReader<impl BufRead>,
    prefixes: &mut HashMap<String, String>,
    base_iri: &Option<Iri<String>>,
    temp_buffer: &mut String,
) -> Result<(), TurtleError> {
    // [4] 	prefixID 	::= 	'@prefix' PNAME_NS IRIREF '.'
    read.consume_many("@prefix".len())?;
    skip_whitespace(read)?;

    let mut prefix = String::default();
    parse_pname_ns(read, &mut prefix)?;
    skip_whitespace(read)?;

    let mut value = String::default();
    parse_iriref_relative(read, &mut value, temp_buffer, base_iri)?;
    skip_whitespace(read)?;

    read.check_is_current(b'.')?;
    read.consume()?;

    prefixes.insert(prefix, value);
    Ok(())
}

pub(crate) fn parse_base(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
    base_iri: &Option<Iri<String>>,
) -> Result<Iri<String>, TurtleError> {
    // [5] 	base 	::= 	'@base' IRIREF '.'
    read.consume_many("@base".len())?;
    skip_whitespace(read)?;

    let result = parse_base_iriref(read, buffer, base_iri)?;
    skip_whitespace(read)?;

    read.check_is_current(b'.')?;
    read.consume()?;

    Ok(result)
}

pub(crate) fn parse_sparql_base(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
    base_iri: &Option<Iri<String>>,
) -> Result<Iri<String>, TurtleError> {
    // [5s] 	sparqlBase 	::= 	"BASE" IRIREF
    read.consume_many("BASE".len())?;
    skip_whitespace(read)?;

    parse_base_iriref(read, buffer, base_iri)
}

fn parse_base_iriref(
    read: &mut LookAheadByteReader<impl BufRead>,
    temp_buffer: &mut String,
    base_iri: &Option<Iri<String>>,
) -> Result<Iri<String>, TurtleError> {
    //TODO: avoid double parsing
    let mut buffer = String::default();
    parse_iriref_relative(read, &mut buffer, temp_buffer, base_iri)?;
    let result = Iri::parse(buffer.clone())
        .map_err(|error| read.parse_error(TurtleErrorKind::InvalidIri { iri: buffer, error }))?;
    temp_buffer.clear();
    Ok(result)
}

fn parse_sparql_prefix(
    read: &mut LookAheadByteReader<impl BufRead>,
    prefixes: &mut HashMap<String, String>,
    base_iri: &Option<Iri<String>>,
    temp_buffer: &mut String,
) -> Result<(), TurtleError> {
    // [6s] 	sparqlPrefix 	::= 	"PREFIX" PNAME_NS IRIREF
    read.consume_many("PREFIX".len())?;
    skip_whitespace(read)?;

    let mut prefix = String::default();
    parse_pname_ns(read, &mut prefix)?;
    skip_whitespace(read)?;

    let mut value = String::default();
    parse_iriref_relative(read, &mut value, temp_buffer, base_iri)?;
    skip_whitespace(read)?;

    prefixes.insert(prefix, value);
    Ok(())
}

fn parse_triples<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [6] 	triples 	::= 	subject predicateObjectList | blankNodePropertyList predicateObjectList?
    match parser.read.current() {
        Some(b'[') if !is_followed_by_space_and_closing_bracket(&mut parser.read)? => {
            let id = parse_blank_node_property_list(parser, on_triple)?;
            parser.triple_alloc.try_push_subject(|b| {
                b.push_str(id.as_ref());
                Ok(Subject::from(BlankNode { id: b }))
            })?;
            skip_whitespace(&mut parser.read)?;
            if parser.read.current() != Some(b'.') && parser.read.current() != Some(b'}') {
                parse_predicate_object_list(parser, on_triple)?;
            }
        }
        _ => {
            parse_subject(parser, on_triple)?;
            skip_whitespace(&mut parser.read)?;
            parse_predicate_object_list(parser, on_triple)?;
        }
    }

    parser.triple_alloc.pop_subject();
    Ok(())
}

fn parse_predicate_object_list<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [7] 	predicateObjectList 	::= 	verb objectList (';' (verb objectList)?)*
    loop {
        parse_verb(parser)?;
        skip_whitespace(&mut parser.read)?;

        parse_object_list(parser, on_triple)?;
        skip_whitespace(&mut parser.read)?;

        parser.triple_alloc.pop_predicate();
        if parser.read.current() != Some(b';') {
            return Ok(());
        }
        while parser.read.current() == Some(b';') {
            parser.read.consume()?;
            skip_whitespace(&mut parser.read)?;
        }
        match parser.read.current() {
            Some(b'.') | Some(b']') | Some(b'}') | None => return Ok(()),
            Some(b'|') => return Ok(()),
            _ => (), //continue
        }
    }
}

fn parse_object_list<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    // [8] 	objectList 	::= 	object (',' object)*
    // or, for RDF-star
    // [8] 	objectList 	::= 	object annotation? ( ',' object annotation? )*
    // [30t] 	annotation 	::= 	'{|' predicateObjectList '|}'
    loop {
        parse_object(parser, on_triple)?;
        skip_whitespace(&mut parser.read)?;

        if parser.read.current() == Some(b'{') {
            parser.read.check_is_next(b'|')?;
            parser.read.consume_many(2)?;
            skip_whitespace(&mut parser.read)?;

            parser.triple_alloc.push_triple_start();
            parser.triple_alloc.push_subject_triple();
            parse_predicate_object_list(parser, on_triple)?;

            parser.read.check_is_current(b'|')?;
            parser.read.check_is_next(b'}')?;
            parser.read.consume_many(2)?;
            skip_whitespace(&mut parser.read)?;
            parser.triple_alloc.pop_annotation_triple();
        }

        parser.triple_alloc.pop_object();
        if parser.read.current() != Some(b',') {
            return Ok(());
        }
        parser.read.consume()?;
        skip_whitespace(&mut parser.read)?;
    }
}

fn parse_verb(parser: &mut TurtleParser<impl BufRead>) -> Result<(), TurtleError> {
    // [9] 	verb 	::= 	predicate | 'a'
    if parser.read.current() == Some(b'a') {
        match parser.read.next()? {
            // We check that it is not a prefixed URI
            Some(c) if is_possible_pn_chars_ascii(c) || c == b'.' || c == b':' || c > MAX_ASCII => {
                parse_predicate(parser)
            }
            _ => {
                parser.read.consume()?;
                parser
                    .triple_alloc
                    .try_push_predicate(|_| Ok(NamedNode { iri: RDF_TYPE }))
            }
        }
    } else {
        parse_predicate(parser)
    }
}

fn parse_subject<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    //[10] 	subject 	::= 	iri | BlankNode | collection
    match parser.read.current() {
        Some(b'_') | Some(b'[') => {
            let TurtleParser {
                read,
                bnode_id_generator,
                triple_alloc,
                ..
            } = parser;
            triple_alloc.try_push_subject(|b| {
                parse_blank_node(read, b, bnode_id_generator).map(Subject::from)
            })?;
        }
        Some(b'(') => {
            let collec = parse_collection(parser, on_triple)?;
            parser
                .triple_alloc
                .try_push_subject(|b| allocate_collection(collec, b))?;
        }
        _ => {
            if parser.read.required_current()? == b'<' && parser.read.required_next()? == b'<' {
                parse_embedded_triple(parser)?;
                parser.triple_alloc.push_subject_triple();
            } else {
                let TurtleParser {
                    read,
                    base_iri,
                    prefixes,
                    triple_alloc,
                    temp_buf,
                    ..
                } = parser;
                triple_alloc.try_push_subject(|b| {
                    parse_iri(read, b, temp_buf, base_iri, prefixes).map(Subject::from)
                })?;
            }
        }
    };
    Ok(())
}

fn parse_predicate(parser: &mut TurtleParser<impl BufRead>) -> Result<(), TurtleError> {
    //[11] 	predicate 	::= 	iri
    let TurtleParser {
        read,
        base_iri,
        prefixes,
        triple_alloc,
        temp_buf,
        ..
    } = parser;
    triple_alloc.try_push_predicate(|b| parse_iri(read, b, temp_buf, base_iri, prefixes))
}

fn parse_object<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<(), E> {
    //[12] 	object 	::= 	iri | BlankNode | collection | blankNodePropertyList | literal

    match parser.read.required_current()? {
        b'<' => {
            if parser.read.required_next()? == b'<' {
                parse_embedded_triple(parser)?;
                parser.triple_alloc.push_object_triple();
            } else {
                let TurtleParser {
                    read,
                    base_iri,
                    triple_alloc,
                    temp_buf,
                    ..
                } = parser;
                triple_alloc.try_push_object(|b, _| {
                    parse_iriref_relative(read, b, temp_buf, base_iri).map(Term::from)
                })?;
            }
        }
        b'(' => {
            let collec = parse_collection(parser, on_triple)?;
            parser
                .triple_alloc
                .try_push_object(|b, _| allocate_collection(collec, b).map(Term::from))?;
        }
        b'[' if !is_followed_by_space_and_closing_bracket(&mut parser.read)? => {
            let id = parse_blank_node_property_list(parser, on_triple)?;
            parser.triple_alloc.try_push_object(|b, _| {
                b.push_str(id.as_ref());
                Ok(Term::from(BlankNode { id: b }))
            })?;
        }
        b'_' | b'[' => {
            let TurtleParser {
                read,
                bnode_id_generator,
                triple_alloc,
                ..
            } = parser;
            triple_alloc.try_push_object(|b, _| {
                parse_blank_node(read, b, bnode_id_generator).map(Term::from)
            })?;
        }
        b'"' | b'\'' => {
            let TurtleParser {
                read,
                base_iri,
                prefixes,
                triple_alloc,
                temp_buf,
                ..
            } = parser;
            triple_alloc.try_push_object(|b1, b2| {
                parse_rdf_literal(read, b1, b2, temp_buf, base_iri, prefixes).map(Term::from)
            })?;
        }
        b'+' | b'-' | b'.' | b'0'..=b'9' => {
            let TurtleParser {
                read, triple_alloc, ..
            } = parser;
            triple_alloc.try_push_object(|b, _| parse_numeric_literal(read, b).map(Term::from))?;
        }
        _ => {
            let TurtleParser {
                read, triple_alloc, ..
            } = parser;
            if read.starts_with(b"true") || read.starts_with(b"false") {
                triple_alloc
                    .try_push_object(|b, _| parse_boolean_literal(read, b).map(Term::from))?;
            } else {
                let TurtleParser {
                    read,
                    prefixes,
                    triple_alloc,
                    ..
                } = parser;
                triple_alloc.try_push_object(|b, _| {
                    parse_prefixed_name(read, b, prefixes).map(Term::from)
                })?;
            }
        }
    };
    on_triple(*parser.triple_alloc.top())
}

fn parse_blank_node_property_list<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<BlankNodeId, E> {
    // [14] 	blankNodePropertyList 	::= 	'[' predicateObjectList ']'
    parser.read.increment_stack_size()?;
    parser.read.check_is_current(b'[')?;
    parser.read.consume()?;
    skip_whitespace(&mut parser.read)?;

    let id = parser.bnode_id_generator.generate();
    parser.triple_alloc.push_triple_start();
    parser.triple_alloc.try_push_subject(|b| {
        b.push_str(id.as_ref());
        Ok(Subject::from(BlankNode { id: b }))
    })?;

    loop {
        parse_predicate_object_list(parser, on_triple)?;
        skip_whitespace(&mut parser.read)?;

        if parser.read.current() == Some(b']') {
            parser.read.consume()?;
            break;
        }
    }

    parser.triple_alloc.pop_subject();
    parser.triple_alloc.pop_top_empty_triple();
    parser.read.decrement_stack_size();
    Ok(id)
}

fn parse_collection<E: From<TurtleError>>(
    parser: &mut TurtleParser<impl BufRead>,
    on_triple: &mut impl FnMut(Triple<'_>) -> Result<(), E>,
) -> Result<Option<BlankNodeId>, E> {
    // [15] 	collection 	::= 	'(' object* ')'
    parser.read.increment_stack_size()?;
    parser.read.check_is_current(b'(')?;
    parser.read.consume()?;
    let mut root: Option<BlankNodeId> = None;
    loop {
        skip_whitespace(&mut parser.read)?;

        if parser.read.current().is_none() {
            return Ok(parser.read.unexpected_char_error()?);
        } else if parser.read.current() != Some(b')') {
            let new = parser.bnode_id_generator.generate();
            if root == None {
                root = Some(new);
                parser.triple_alloc.push_triple_start();
            } else {
                parser
                    .triple_alloc
                    .try_push_predicate(|_| Ok(NamedNode { iri: RDF_REST }))?;
                parser.triple_alloc.try_push_object(|b, _| {
                    b.push_str(new.as_ref());
                    Ok(Term::from(BlankNode { id: b }))
                })?;
                on_triple(*parser.triple_alloc.top())?;
                parser.triple_alloc.pop_object();
                parser.triple_alloc.pop_predicate();
                parser.triple_alloc.pop_subject();
            }

            parser.triple_alloc.try_push_subject(|b| {
                b.push_str(new.as_ref());
                Ok(Subject::from(BlankNode { id: b }))
            })?;
            parser
                .triple_alloc
                .try_push_predicate(|_| Ok(NamedNode { iri: RDF_FIRST }))?;
            parse_object(parser, on_triple)?;
            parser.triple_alloc.pop_object();
            parser.triple_alloc.pop_predicate();
        } else {
            // trailing ')'
            parser.read.consume()?;
            if root.is_some() {
                parser
                    .triple_alloc
                    .try_push_predicate(|_| Ok(NamedNode { iri: RDF_REST }))?;
                parser
                    .triple_alloc
                    .try_push_object(|_, _| Ok(Term::from(NamedNode { iri: RDF_NIL })))?;
                on_triple(*parser.triple_alloc.top())?;
                parser.triple_alloc.pop_top_triple();
            }
            parser.read.decrement_stack_size();
            return Ok(root);
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn allocate_collection(
    collection: Option<BlankNodeId>,
    buffer: &mut String,
) -> Result<Subject<'_>, TurtleError> {
    match collection {
        Some(id) => {
            buffer.push_str(id.as_ref());
            Ok(BlankNode { id: buffer }.into())
        }
        None => Ok(NamedNode { iri: RDF_NIL }.into()),
    }
}

pub(crate) fn parse_numeric_literal<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
) -> Result<Literal<'a>, TurtleError> {
    // [16] 	NumericLiteral 	::= 	INTEGER | DECIMAL | DOUBLE
    // [19] 	INTEGER 	::= 	[+-]? [0-9]+
    // [20] 	DECIMAL 	::= 	[+-]? [0-9]* '.' [0-9]+
    // [21] 	DOUBLE 	::= 	[+-]? ([0-9]+ '.' [0-9]* EXPONENT | '.' [0-9]+ EXPONENT | [0-9]+ EXPONENT)
    // merged [+-] [0-9]* ('.' [0-9]*)? EXPONENT?
    let c = read.required_current()?;
    match c {
        b'+' | b'-' => {
            buffer.push(char::from(c));
            read.consume()?
        }
        _ => (),
    }

    // We read the digits before .
    let mut count_before: usize = 0;
    while let Some(c) = read.current() {
        match c {
            b'0'..=b'9' => {
                buffer.push(char::from(c));
                read.consume()?;
                count_before += 1;
            }
            _ => break,
        }
    }

    // We read the digits after .
    let count_after = if read.current() == Some(b'.') {
        //We check if it is not the end of a statement

        let stop = match read.next()? {
            Some(c) => !matches!(c, b'0'..=b'9' | b'e' | b'E'),
            None => true,
        };
        if stop {
            return if count_before > 0 {
                Ok(Literal::Typed {
                    value: buffer,
                    datatype: NamedNode { iri: XSD_INTEGER },
                })
            } else {
                read.unexpected_char_error()
            };
        }

        buffer.push('.');
        let mut count_after = 0;

        read.consume()?;
        while let Some(c) = read.current() {
            match c {
                b'0'..=b'9' => {
                    buffer.push(char::from(c));
                    read.consume()?;
                    count_after += 1;
                }
                _ => break,
            }
        }
        Some(count_after)
    } else {
        None
    };

    // End
    let datatype = match read.current() {
        Some(b'e') | Some(b'E') => {
            if count_before > 0 || count_after.unwrap_or(0) > 0 {
                parse_exponent(read, buffer)?;
                XSD_DOUBLE
            } else {
                return read.unexpected_char_error();
            }
        }
        _ => {
            if count_after.is_none() && count_before > 0 {
                XSD_INTEGER
            } else if count_after != None && count_after != Some(0) {
                XSD_DECIMAL
            } else {
                return read.unexpected_char_error();
            }
        }
    };
    Ok(Literal::Typed {
        value: buffer,
        datatype: NamedNode { iri: datatype },
    })
}

pub(crate) fn parse_rdf_literal<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
    annotation_buffer: &'a mut String,
    temp_buffer: &mut String,
    base_iri: &Option<Iri<String>>,
    prefixes: &HashMap<String, String>,
) -> Result<Literal<'a>, TurtleError> {
    // [128s] 	RDFLiteral 	::= 	String (LANGTAG | '^^' iri)?
    parse_string(read, buffer)?;
    skip_whitespace(read)?;

    match read.current() {
        Some(b'@') => {
            parse_langtag(read, annotation_buffer)?;
            Ok(Literal::LanguageTaggedString {
                value: buffer,
                language: annotation_buffer,
            })
        }
        Some(b'^') => {
            read.consume()?;
            read.check_is_current(b'^')?;
            read.consume()?;
            skip_whitespace(read)?;
            parse_iri(read, annotation_buffer, temp_buffer, base_iri, prefixes)?;
            Ok(Literal::Typed {
                value: buffer,
                datatype: NamedNode {
                    iri: annotation_buffer,
                },
            })
        }
        _ => Ok(Literal::Simple { value: buffer }),
    }
}

pub(crate) fn parse_boolean_literal<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
) -> Result<Literal<'a>, TurtleError> {
    if read.starts_with(b"true") {
        read.consume_many("true".len())?;
        buffer.push_str("true");
    } else if read.starts_with(b"false") {
        read.consume_many("false".len())?;
        buffer.push_str("false");
    } else {
        return read.unexpected_char_error();
    }
    Ok(Literal::Typed {
        value: buffer,
        datatype: NamedNode { iri: XSD_BOOLEAN },
    })
}

fn parse_string(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    match read.current() {
        Some(b'"') => {
            if read.starts_with(b"\"\"\"") {
                parse_string_literal_long_quote(read, buffer)
            } else {
                parse_string_literal_quote(read, buffer)
            }
        }
        Some(b'\'') => {
            if read.starts_with(b"'''") {
                parse_string_literal_long_single_quote(read, buffer)
            } else {
                parse_string_literal_single_quote(read, buffer)
            }
        }
        _ => read.unexpected_char_error(),
    }
}

pub(crate) fn parse_iri<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
    temp_buffer: &mut String,
    base_iri: &Option<Iri<String>>,
    prefixes: &HashMap<String, String>,
) -> Result<NamedNode<'a>, TurtleError> {
    // [135s] 	iri 	::= 	IRIREF | PrefixedName
    if read.current() == Some(b'<') {
        parse_iriref_relative(read, buffer, temp_buffer, base_iri)
    } else {
        parse_prefixed_name(read, buffer, prefixes)
    }
}

pub(crate) fn parse_prefixed_name<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
    prefixes: &HashMap<String, String>,
) -> Result<NamedNode<'a>, TurtleError> {
    // [136s] 	PrefixedName 	::= 	PNAME_LN | PNAME_NS
    // It could be written: PNAME_NS PN_LOCAL?

    // PNAME_NS
    parse_pname_ns(read, buffer)?;
    if let Some(value) = prefixes.get(buffer.as_str()) {
        buffer.clear();
        buffer.push_str(value);
    } else {
        return Err(read.parse_error(TurtleErrorKind::UnknownPrefix(buffer.clone())));
    }

    // [168s] 	PN_LOCAL 	::= 	(PN_CHARS_U | ':' | [0-9] | PLX) ((PN_CHARS | '.' | ':' | PLX)* (PN_CHARS | ':' | PLX))?
    if let Some(c) = read.current() {
        match c {
            b'\\' => parse_pn_local_esc(read, buffer)?,
            b'%' => parse_percent(read, buffer)?,
            b':' | b'0'..=b'9' => buffer.push(char::from(c)),
            c if is_possible_pn_chars_u_ascii(c) => buffer.push(char::from(c)),
            _ => {
                let c = read_utf8_char(read)?;
                if is_possible_pn_chars_u_unicode(c) {
                    buffer.push(c)
                } else {
                    return Ok(NamedNode { iri: buffer });
                }
            }
        }
    } else {
        return Ok(NamedNode { iri: buffer });
    }

    loop {
        read.consume()?;
        match read.current() {
            Some(b'.') => {
                if has_future_char_valid_pname_local(read)? {
                    buffer.push('.')
                } else {
                    break;
                }
            }
            Some(b'\\') => parse_pn_local_esc(read, buffer)?,
            Some(b'%') => parse_percent(read, buffer)?,
            Some(b':') => buffer.push(':'),
            Some(c) if is_possible_pn_chars_ascii(c) => buffer.push(char::from(c)),
            _ => {
                let c = read_utf8_char(read)?;
                if is_possible_pn_chars_unicode(c) {
                    buffer.push(c)
                } else {
                    break;
                }
            }
        }
    }
    Ok(NamedNode { iri: buffer })
}

fn has_future_char_valid_pname_local(
    read: &mut LookAheadByteReader<impl BufRead>,
) -> Result<bool, TurtleError> {
    let mut i = 1;
    loop {
        match read.ahead(i)? {
            Some(b':') | Some(b'%') | Some(b'\\') => return Ok(true),
            Some(c) if c > MAX_ASCII || is_possible_pn_chars_ascii(c) => return Ok(true),
            Some(b'.') => (),
            _ => return Ok(false),
        }
        i += 1;
    }
}

pub(crate) fn parse_blank_node<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
    bnode_id_generator: &mut BlankNodeIdGenerator,
) -> Result<BlankNode<'a>, TurtleError> {
    // [137s] 	BlankNode 	::= 	BLANK_NODE_LABEL | ANON
    match read.current() {
        Some(b'_') => {
            parse_blank_node_label(read, buffer)?;
            bnode_id_generator.disambiguate(buffer);
        }
        Some(b'[') => {
            parse_anon(read, buffer, bnode_id_generator)?;
        }
        _ => read.unexpected_char_error()?,
    }
    Ok(BlankNode { id: buffer })
}

pub(crate) fn parse_pname_ns(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [139s] 	PNAME_NS 	::= 	PN_PREFIX? ':'
    parse_pn_prefix(read, buffer)?;
    if read.current() == Some(b':') {
        read.consume()?;
        Ok(())
    } else {
        read.unexpected_char_error()
    }
}

fn parse_exponent(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [154s] 	EXPONENT 	::= 	[eE] [+-]? [0-9]+
    let c = read.required_current()?;
    match c {
        b'e' | b'E' => buffer.push(char::from(c)),
        _ => read.unexpected_char_error()?,
    };
    read.consume()?;

    if let Some(c) = read.current() {
        match c {
            b'+' | b'-' => {
                buffer.push(char::from(c));
                read.consume()?
            }
            _ => (),
        }
    }

    match read.required_current()? {
        c @ b'0'..=b'9' => buffer.push(char::from(c)),
        _ => read.unexpected_char_error()?,
    }

    loop {
        read.consume()?;
        if let Some(c) = read.current() {
            match c {
                b'0'..=b'9' => buffer.push(char::from(c)),
                _ => return Ok(()),
            }
        } else {
            return Ok(());
        }
    }
}

fn parse_string_literal_single_quote(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [23] 	STRING_LITERAL_SINGLE_QUOTE 	::= 	"'" ([^#x27#x5C#xA#xD] | ECHAR | UCHAR)* "'" /* #x27=' #x5C=\ #xA=new line #xD=carriage return */
    parse_string_literal_quote_inner(read, buffer, b'\'')
}

fn parse_string_literal_long_single_quote(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [24] 	STRING_LITERAL_LONG_SINGLE_QUOTE 	::= 	"'''" (("'" | "''")? ([^'\] | ECHAR | UCHAR))* "'''"
    parse_string_literal_long_quote_inner(read, buffer, b'\'')
}

fn parse_string_literal_long_quote(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [25] 	STRING_LITERAL_LONG_QUOTE 	::= 	'"""' (('"' | '""')? ([^"\] | ECHAR | UCHAR))* '"""'
    parse_string_literal_long_quote_inner(read, buffer, b'"')
}

fn parse_string_literal_long_quote_inner(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
    quote: u8,
) -> Result<(), TurtleError> {
    let prefix = [quote; 3];
    read.consume_many(2)?;
    loop {
        read.consume()?;
        match read.required_current()? {
            c if c == quote && read.starts_with(&prefix) => {
                read.consume_many(3)?;
                return Ok(());
            }
            b'\\' => parse_echar_or_uchar(read, buffer)?,
            c => buffer.push(if c <= 0x7F {
                char::from(c) //optimization to avoid UTF-8 decoding
            } else {
                read_utf8_char(read)?
            }),
        }
    }
}

fn parse_anon(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
    bnode_id_generator: &mut BlankNodeIdGenerator,
) -> Result<(), TurtleError> {
    read.check_is_current(b'[')?;
    read.consume()?;
    skip_whitespace(read)?;

    read.check_is_current(b']')?;
    read.consume()?;

    buffer.push_str(bnode_id_generator.generate().as_ref());
    Ok(())
}

fn parse_pn_prefix(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [167s] 	PN_PREFIX 	::= 	PN_CHARS_BASE ((PN_CHARS | '.')* PN_CHARS)?
    match read.current() {
        Some(c) if c <= MAX_ASCII && is_possible_pn_chars_base_ascii(c) => {
            buffer.push(char::from(c))
        }
        _ => {
            let c = read_utf8_char(read)?;
            if is_possible_pn_chars_base_unicode(c) {
                buffer.push(c)
            } else {
                return Ok(()); //PN_PREFIX is always optional
            }
        }
    }

    loop {
        read.consume()?;
        match read.current() {
            Some(b'.') => match read.next()? {
                Some(c) if is_possible_pn_chars_ascii(c) || c > MAX_ASCII => buffer.push('.'),
                _ => {
                    return Ok(());
                }
            },
            Some(c) if c <= MAX_ASCII && is_possible_pn_chars_ascii(c) => {
                buffer.push(char::from(c))
            }
            _ => {
                let c = read_utf8_char(read)?;
                if is_possible_pn_chars_unicode(c) {
                    buffer.push(c)
                } else {
                    return Ok(());
                }
            }
        }
    }
}

fn parse_percent(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [170s] 	PERCENT 	::= 	'%' HEX HEX
    read.check_is_current(b'%')?;
    buffer.push('%');
    read.consume()?;
    parse_hex(read, buffer)?;
    read.consume()?;
    parse_hex(read, buffer)?;
    Ok(())
}

fn parse_hex(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [171s] 	HEX 	::= 	[0-9] | [A-F] | [a-f]
    let c = read.required_current()?;
    match c {
        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
            buffer.push(char::from(c));
            Ok(())
        }
        _ => read.unexpected_char_error(),
    }
}

fn parse_pn_local_esc(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [172s] 	PN_LOCAL_ESC 	::= 	'\' ('_' | '~' | '.' | '-' | '!' | '$' | '&' | "'" | '(' | ')' | '*' | '+' | ',' | ';' | '=' | '/' | '?' | '#' | '@' | '%')
    read.check_is_current(b'\\')?;
    read.consume()?;
    let c = read.required_current()?;
    match c {
        b'_' | b'~' | b'.' | b'-' | b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+'
        | b',' | b';' | b'=' | b'/' | b'?' | b'#' | b'@' | b'%' => {
            buffer.push(char::from(c));
            Ok(())
        }
        _ => read.unexpected_char_error(),
    }
}

pub(crate) fn skip_whitespace(
    read: &mut LookAheadByteReader<impl BufRead>,
) -> Result<(), TurtleError> {
    loop {
        match read.current() {
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => read.consume()?,
            Some(b'#') => {
                while read.current() != Some(b'\r')
                    && read.current() != Some(b'\n')
                    && read.current() != None
                {
                    read.consume()?;
                }
            }
            _ => return Ok(()),
        }
    }
}

pub(crate) fn is_followed_by_space_and_closing_bracket(
    read: &mut LookAheadByteReader<impl BufRead>,
) -> Result<bool, TurtleError> {
    for i in 1.. {
        match read.ahead(i)? {
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => (),
            Some(b']') => return Ok(true),
            _ => return Ok(false),
        }
    }
    Ok(false)
}

fn on_triple_in_graph<'a, E>(
    on_quad: &'a mut impl FnMut(Quad<'_>) -> Result<(), E>,
    graph_name: Option<GraphName<'a>>,
) -> impl FnMut(Triple<'_>) -> Result<(), E> + 'a {
    move |t: Triple<'_>| {
        on_quad(Quad {
            subject: t.subject,
            predicate: t.predicate,
            object: t.object,
            graph_name,
        })
    }
}

pub(crate) fn parse_embedded_triple(
    parser: &mut TurtleParser<impl BufRead>,
) -> Result<(), TurtleError> {
    // [27t] 	embTriple 	::= 	'<<' embSubject verb embObject '>>'
    parser.read.increment_stack_size()?;
    parser.read.consume_many(2)?;
    skip_whitespace(&mut parser.read)?;

    parser.triple_alloc.push_triple_start();

    parse_emb_subject(parser)?;
    skip_whitespace(&mut parser.read)?;

    parse_verb(parser)?;
    skip_whitespace(&mut parser.read)?;

    parse_emb_object(parser)?;
    skip_whitespace(&mut parser.read)?;

    parser.read.check_is_current(b'>')?;
    parser.read.check_is_next(b'>')?;
    parser.read.consume_many(2)?;
    parser.read.decrement_stack_size();
    Ok(())
}

pub(crate) fn parse_emb_subject(
    parser: &mut TurtleParser<impl BufRead>,
) -> Result<(), TurtleError> {
    // [28t] 	embSubject 	::= 	iri | BlankNode | embTriple
    match parser.read.current() {
        Some(b'<') => {
            if parser.read.required_next()? == b'<' {
                parse_embedded_triple(parser)?;
                parser.triple_alloc.push_subject_triple();
                Ok(())
            } else {
                let TurtleParser {
                    read,
                    base_iri,
                    triple_alloc,
                    temp_buf,
                    ..
                } = parser;
                triple_alloc.try_push_subject(|b| {
                    parse_iriref_relative(read, b, temp_buf, base_iri).map(Subject::from)
                })
            }
        }
        Some(b'_') | Some(b'[') => {
            let TurtleParser {
                read,
                bnode_id_generator,
                triple_alloc,
                ..
            } = parser;
            triple_alloc.try_push_subject(|b| {
                parse_blank_node(read, b, bnode_id_generator).map(Subject::from)
            })
        }
        _ => {
            let TurtleParser {
                read,
                prefixes,
                triple_alloc,
                ..
            } = parser;
            triple_alloc
                .try_push_subject(|b| parse_prefixed_name(read, b, prefixes).map(Subject::from))
        }
    }
}

pub(crate) fn parse_emb_object(parser: &mut TurtleParser<impl BufRead>) -> Result<(), TurtleError> {
    // [29t] 	embObject 	::= 	iri | BlankNode | literal | embTriple
    match parser.read.required_current()? {
        b'<' => {
            if parser.read.required_next()? == b'<' {
                parse_embedded_triple(parser)?;
                parser.triple_alloc.push_object_triple();
                Ok(())
            } else {
                let TurtleParser {
                    read,
                    base_iri,
                    triple_alloc,
                    temp_buf,
                    ..
                } = parser;
                triple_alloc.try_push_object(|b, _| {
                    parse_iriref_relative(read, b, temp_buf, base_iri).map(Term::from)
                })
            }
        }
        b'_' | b'[' => {
            let TurtleParser {
                read,
                bnode_id_generator,
                triple_alloc,
                ..
            } = parser;
            triple_alloc.try_push_object(|b, _| {
                parse_blank_node(read, b, bnode_id_generator).map(Term::from)
            })
        }
        b'"' | b'\'' => {
            let TurtleParser {
                read,
                base_iri,
                prefixes,
                triple_alloc,
                temp_buf,
                ..
            } = parser;
            triple_alloc.try_push_object(|b1, b2| {
                parse_rdf_literal(read, b1, b2, temp_buf, base_iri, prefixes).map(Term::from)
            })
        }
        b'+' | b'-' | b'.' | b'0'..=b'9' => {
            let TurtleParser {
                read, triple_alloc, ..
            } = parser;
            triple_alloc.try_push_object(|b, _| parse_numeric_literal(read, b).map(Term::from))
        }
        _ => {
            let TurtleParser {
                read, triple_alloc, ..
            } = parser;
            if read.starts_with(b"true") || read.starts_with(b"false") {
                triple_alloc.try_push_object(|b, _| parse_boolean_literal(read, b).map(Term::from))
            } else {
                let TurtleParser {
                    read,
                    prefixes,
                    triple_alloc,
                    ..
                } = parser;
                triple_alloc
                    .try_push_object(|b, _| parse_prefixed_name(read, b, prefixes).map(Term::from))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn issue_46() -> Result<(), TurtleError> {
        let bnid = BlankNodeIdGenerator::default().generate();

        let ttl = format!(
            r#"PREFIX : <tag:>
            :alice :knows [ :name "bob" ].
            _:{} :name "charlie".
            "#,
            bnid.as_ref()
        );

        let mut blank_subjects = vec![];
        TurtleParser::_new(std::io::Cursor::new(&ttl), None).parse_all(&mut |t| -> Result<
            (),
            TurtleError,
        > {
            if let Subject::BlankNode(b) = t.subject {
                blank_subjects.push(b.id.to_string());
            }
            Ok(())
        })?;
        assert_eq!(blank_subjects.len(), 2);
        assert_ne!(&blank_subjects[0], &blank_subjects[1]);
        Ok(())
    }
}
