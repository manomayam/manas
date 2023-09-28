//! I define [`TripleAllocator`]

#![allow(unsafe_code)]
use std::mem::transmute;

use rio_api::model::*;

use super::utils::StringBufferStack;

/// A stack allocator for storing RDF and RDF* triples.
///
/// # Implementation
/// This type uses `&'static str` internally to reference text that it allocates.
/// It therefore contains unsafe code to cheat the borrow checker,
/// but ensures that the referenced data lives as long as the referencing struct.
pub struct TripleAllocator {
    incomplete_stack: Vec<Triple<'static>>,
    incomplete_len: usize,
    #[allow(clippy::vec_box)]
    complete_stack: Vec<Box<Triple<'static>>>,
    complete_len: usize,
    string_stack: StringBufferStack,
}

impl TripleAllocator {
    pub fn _new() -> TripleAllocator {
        TripleAllocator {
            incomplete_stack: Vec::with_capacity(1),
            incomplete_len: 0,
            complete_stack: Vec::with_capacity(1),
            complete_len: 0,
            string_stack: StringBufferStack::_with_capacity(4),
        }
    }

    pub fn top(&self) -> &Triple<'_> {
        debug_assert!(self.complete_len > 0);
        &self.complete_stack[self.complete_len - 1]
    }

    pub fn _top_quad<'s>(&'s self, graph_name: Option<GraphName<'s>>) -> Quad<'s> {
        debug_assert!(self.complete_len > 0);
        let Triple {
            subject,
            predicate,
            object,
        } = *self.top();
        Quad {
            subject,
            predicate,
            object,
            graph_name,
        }
    }

    pub fn push_triple_start(&mut self) {
        if self.incomplete_len == self.incomplete_stack.len() {
            self.incomplete_stack.push(Triple {
                subject: DUMMY_IRI.into(),
                predicate: DUMMY_IRI,
                object: DUMMY_IRI.into(),
            })
        }
        #[cfg(debug_assertions)] // to ensure that dummy() assertions work
        {
            self.incomplete_stack[self.incomplete_len] = Triple {
                subject: DUMMY_IRI.into(),
                predicate: DUMMY_IRI,
                object: DUMMY_IRI.into(),
            }
        }
        self.incomplete_len += 1;
    }

    /// Push an atomic term, produced by `subject_factory`, as the subject of the current triple.
    ///
    /// # Pre-condition
    /// In standard RDF, any subject is acceptable.
    /// In RDF* embedded triples [`Subject::Triple`] are *not allowed*.
    /// For adding an embedded triple, use [`TripleAllocator::push_subject_triple`] instead.
    pub fn try_push_subject<E, F>(&mut self, subject_factory: F) -> Result<(), E>
    where
        F: FnOnce(&mut String) -> Result<Subject<'_>, E>,
    {
        debug_assert!(dummy(self.current().subject));
        let buffer = self.string_stack.push();
        let subject = subject_factory(buffer)?;
        debug_assert!(matches!(
            subject,
            Subject::NamedNode(_) | Subject::BlankNode(_)
        ));
        let subject: Subject<'static> = unsafe { transmute(subject) };
        // The unsafe code above changes the lifetime parameter of subject to `'static`.
        // This is ok because:
        // * we will only expose it with a shorter lifetime, and
        // * this implementation guarantees that the pointed `str` lives as long as the subject
        self.current().subject = subject;
        Ok(())
    }

    /// Push an atomic term, produced by `predicate_factory`, as the predicate of the current triple.
    pub fn try_push_predicate<E, F>(&mut self, predicate_factory: F) -> Result<(), E>
    where
        F: FnOnce(&mut String) -> Result<NamedNode<'_>, E>,
    {
        debug_assert!(!dummy(self.current().subject));
        debug_assert!(dummy(self.current().predicate));
        let buffer = self.string_stack.push();
        let predicate = predicate_factory(buffer)?;
        let predicate: NamedNode<'static> = unsafe { transmute(predicate) };
        // The unsafe code above changes the lifetime parameter of predicate to `'static`.
        // This is ok because:
        // * we will only expose it with a shorter lifetime, and
        // * this implementation guarantees that the pointed `str` lives as long as the predicate
        self.current().predicate = predicate;
        Ok(())
    }

    /// Push an atomic term, produced by `object_factory`, as the object of the current triple.
    ///
    /// # Pre-condition
    /// In standard RDF, any object is acceptable.
    /// In RDF* embedded triples [`Subject::Triple`] are *not allowed*.
    /// For adding an embedded triple, use [`TripleAllocator::push_object_triple`] instead.
    pub fn try_push_object<E, F>(&mut self, object_factory: F) -> Result<(), E>
    where
        F: for<'x> FnOnce(&'x mut String, &'x mut String) -> Result<Term<'x>, E>,
    {
        debug_assert!(!dummy(self.current().predicate));
        debug_assert!(dummy(self.current().object));
        let buffers = self.string_stack.push2();
        let object = object_factory(buffers.0, buffers.1)?;
        debug_assert!(matches!(
            object,
            Term::NamedNode(_) | Term::BlankNode(_) | Term::Literal(_)
        ));
        let object: Term<'static> = unsafe { transmute(object) };
        // The unsafe code above changes the lifetime parameter of object to `'static`.
        // This is ok because:
        // * we will only expose it with a shorter lifetime, and
        // * this implementation guarantees that the pointed `str` lives as long as the object
        self.complete_triple(object);
        Ok(())
    }

    /// Use the [top](TripleAllocator::top) triple of this stash as the subject of the current triple.
    pub fn push_subject_triple(&mut self) {
        debug_assert!(dummy(self.current().subject));
        debug_assert!(self.complete_len > 0);
        let triple = &*self.complete_stack[self.complete_len - 1];
        let triple: &'static Triple<'static> = unsafe { transmute(triple) };
        // The unsafe code above changes the lifetime of the ref to `'static`.
        // This is ok because:
        // * we will only expose it with a shorter lifetime, and
        // * this implementation guarantees that the pointed `Triple` lives as long as the `Term` embedding it
        self.current().subject = Subject::Triple(triple);
    }

    /// Use the [top](TripleAllocator::top) triple of this stash as the object of the current triple.
    ///
    /// # Pre-condition
    /// The top triple must not have been pushed already as the subject.
    pub fn push_object_triple(&mut self) {
        debug_assert!(!dummy(self.current().predicate));
        debug_assert!(dummy(self.current().object));
        debug_assert!(self.complete_len > 0);
        let triple = &*self.complete_stack[self.complete_len - 1];

        #[cfg(debug_assertions)] // the subject triple, if any, must not be top()
        debug_assert!(
            match self.incomplete_stack[self.incomplete_len - 1].subject {
                Subject::Triple(s) => {
                    let ptr_s: *const _ = s;
                    ptr_s != triple
                }
                _ => true,
            }
        );

        let triple: &'static Triple<'static> = unsafe { transmute(triple) };
        // The unsafe code above changes the lifetime of the ref to `'static`.
        // This is ok because:
        // * we will only expose it with a shorter lifetime, and
        // * this implementation guarantees that the pointed `Triple` lives as long as the `Term` embedding it
        self.complete_triple(Term::Triple(triple));
    }

    pub fn pop_object(&mut self) {
        debug_assert!(self.complete_len > 0);
        self.complete_len -= 1;
        let inc_triple = *self.complete_stack[self.complete_len];
        if self.incomplete_len == self.incomplete_stack.len() {
            self.incomplete_stack.push(inc_triple)
        } else {
            self.incomplete_stack[self.incomplete_len] = inc_triple;
        }
        self.incomplete_len += 1;

        match inc_triple.object {
            Term::NamedNode(_) | Term::BlankNode(_) | Term::Literal(_) => {
                // we allocate two buffers for any atomic object, even named or blank node
                self.string_stack.pop();
                self.string_stack.pop()
            }
            Term::Triple(_) => self.pop_top_triple(),
        }
        #[cfg(debug_assertions)] // to ensure that dummy() assertions work
        {
            self.current().object = DUMMY_IRI.into();
        }
    }

    pub fn pop_predicate(&mut self) {
        debug_assert!(dummy(self.current().object));
        debug_assert!(!dummy(self.current().predicate));
        self.string_stack.pop();
        #[cfg(debug_assertions)] // to ensure that dummy() assertions work
        {
            self.current().predicate = DUMMY_IRI;
        }
    }

    pub fn pop_subject(&mut self) {
        debug_assert!(dummy(self.current().predicate));
        debug_assert!(!dummy(self.current().subject));
        match self.current().subject {
            Subject::NamedNode(_) | Subject::BlankNode(_) => self.string_stack.pop(),
            Subject::Triple(_) => self.pop_top_triple(),
        }
        #[cfg(debug_assertions)] // to ensure that dummy() assertions work
        {
            self.current().subject = DUMMY_IRI.into();
        }
    }

    #[inline(always)]
    /// Pops the latest complete triple, and recursively pops all its constituent triples.
    /// Equivalent to pop_object, pop_predicate, pop_subject, pop_empty_triple
    pub fn pop_top_triple(&mut self) {
        self.pop_object();
        self.pop_predicate();
        self.pop_subject();
        self.incomplete_len -= 1;
    }

    /// Pops the top-most empty triple, created with push_triple_start,
    /// but with all its components having been popped (or never pushed)
    #[inline(always)]
    pub fn pop_top_empty_triple(&mut self) {
        debug_assert!(self.incomplete_len > 0);
        debug_assert!(dummy(self.current().predicate));
        debug_assert!(dummy(self.current().subject));
        self.incomplete_len -= 1;
    }

    /// Pops the top-most annotation triple, i.e.
    /// a triple on the incomplete stack with only its subject pushed,
    /// and that subject is an embedded triple.
    ///
    /// The goal is to remove this triple *without* freeing the subject triple.
    #[inline(always)]
    pub fn pop_annotation_triple(&mut self) {
        debug_assert!(self.incomplete_len > 0);
        debug_assert!(dummy(self.current().predicate));
        debug_assert!(!dummy(self.current().subject));
        debug_assert!(matches!(self.current().subject, Subject::Triple(_)));
        self.incomplete_len -= 1;
    }

    pub fn _clear(&mut self) {
        self.incomplete_len = 0;
        self.incomplete_stack.clear();
        self.complete_len = 0;
        self.complete_stack.clear();
        self.string_stack._clear();
    }

    fn complete_triple(&mut self, object: Term<'static>) {
        self.incomplete_len -= 1;
        let mut triple = self.incomplete_stack[self.incomplete_len];
        triple.object = object;
        if self.complete_len == self.complete_stack.len() {
            self.complete_stack.push(Box::new(triple))
        } else {
            *self.complete_stack[self.complete_len] = triple;
        }
        self.complete_len += 1;
    }

    fn current(&mut self) -> &mut Triple<'static> {
        debug_assert!(self.incomplete_len > 0);
        &mut self.incomplete_stack[self.incomplete_len - 1]
    }
}

#[cfg(debug_assertions)] // debug assertions need DUMMY to have a static address
static DUMMY_IRI: NamedNode<'static> = NamedNode { iri: "" };
#[cfg(not(debug_assertions))] // otherwise, a const is sufficient
const DUMMY_IRI: NamedNode<'static> = NamedNode { iri: "" };

fn dummy<'a, T: std::fmt::Debug + Into<Term<'a>>>(t: T) -> bool {
    match t.into() {
        Term::NamedNode(n) => {
            let ptr_iri: *const str = n.iri;
            ptr_iri == DUMMY_IRI.iri
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;

    #[cfg(debug_assertions)]
    #[test]
    fn dummy_works() {
        assert!(dummy(DUMMY_IRI));
        let b = "foo".to_string();
        let n = NamedNode { iri: &b[..0] };
        assert!(DUMMY_IRI.iri == n.iri);
        // and yet:
        assert!(!dummy(n));
    }

    fn iri<'a, T: From<NamedNode<'a>>>(
        buffer: &'a mut String,
        value: &str,
    ) -> Result<T, Infallible> {
        buffer.push_str(value);
        Ok(NamedNode { iri: &*buffer }.into())
    }

    fn bn<'a, T: From<BlankNode<'a>>>(
        buffer: &'a mut String,
        value: &str,
    ) -> Result<T, Infallible> {
        buffer.push_str(value);
        Ok(BlankNode { id: &*buffer }.into())
    }

    fn sl<'a, T: From<Literal<'a>>>(buffer: &'a mut String, value: &str) -> Result<T, Infallible> {
        buffer.push_str(value);
        Ok(Literal::Simple { value: &*buffer }.into())
    }

    fn lt<'a, T: From<Literal<'a>>>(
        buffer1: &'a mut String,
        buffer2: &'a mut String,
        value: &str,
        tag: &str,
    ) -> Result<T, Infallible> {
        buffer1.push_str(value);
        buffer2.push_str(tag);
        Ok(Literal::LanguageTaggedString {
            value: &*buffer1,
            language: &*buffer2,
        }
        .into())
    }

    fn dt<'a, T: From<Literal<'a>>>(
        buffer1: &'a mut String,
        buffer2: &'a mut String,
        value: &str,
        dt: &str,
    ) -> Result<T, Infallible> {
        buffer1.push_str(value);
        buffer2.push_str(dt);
        Ok(Literal::Typed {
            value: &*buffer1,
            datatype: NamedNode { iri: &*buffer2 },
        }
        .into())
    }

    #[test]
    fn simple_triple_w_named() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| iri(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b, _| iri(b, "c"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <b> <c>"#);
        Ok(())
    }

    #[test]
    fn simple_triple_w_blank() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| bn(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b, _| bn(b, "c"))?;
        assert_eq!(format!("{}", ta.top()), r#"_:a <b> _:c"#);
        Ok(())
    }

    #[test]
    fn simple_triple_w_simple_lit() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| bn(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b, _| sl(b, "c"))?;
        assert_eq!(format!("{}", ta.top()), r#"_:a <b> "c""#);
        Ok(())
    }

    #[test]
    fn simple_triple_w_lang_lit() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| bn(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b1, b2| lt(b1, b2, "c", "en"))?;
        assert_eq!(format!("{}", ta.top()), r#"_:a <b> "c"@en"#);
        Ok(())
    }

    #[test]
    fn simple_triple_w_typed_lit() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| bn(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b1, b2| dt(b1, b2, "c", "d"))?;
        assert_eq!(format!("{}", ta.top()), r#"_:a <b> "c"^^<d>"#);
        Ok(())
    }

    #[test]
    fn simple_triples_pop() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| iri(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b, _| iri(b, "c"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <b> <c>"#);
        ta.pop_object();
        ta.try_push_object(|b, _| iri(b, "d"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <b> <d>"#);
        ta.pop_object();
        ta.pop_predicate();
        ta.try_push_predicate(|b| iri(b, "e"))?;
        ta.try_push_object(|b, _| iri(b, "f"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <e> <f>"#);
        ta.pop_object();
        ta.try_push_object(|b, _| iri(b, "g"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <e> <g>"#);
        ta.pop_object();
        ta.pop_predicate();
        ta.pop_subject();
        ta.try_push_subject(|b| iri(b, "h"))?;
        ta.try_push_predicate(|b| iri(b, "i"))?;
        ta.try_push_object(|b, _| iri(b, "j"))?;
        assert_eq!(format!("{}", ta.top()), r#"<h> <i> <j>"#);
        Ok(())
    }

    #[test]
    fn simple_triples_stacked() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| iri(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        ta.try_push_object(|b, _| iri(b, "c"))?;
        assert_eq!(format!("{}", ta.top()), r#"<a> <b> <c>"#);
        ta.push_triple_start();
        ta.try_push_subject(|b| iri(b, "d"))?;
        ta.try_push_predicate(|b| iri(b, "e"))?;
        ta.try_push_object(|b, _| iri(b, "f"))?;
        assert_eq!(format!("{}", ta.top()), r#"<d> <e> <f>"#);
        ta.pop_top_triple();
        assert_eq!(format!("{}", ta.top()), r#"<a> <b> <c>"#);
        ta.pop_top_triple();
        assert_eq!(ta.complete_len, 0);
        assert_eq!(ta.incomplete_len, 0);
        Ok(())
    }

    #[test]
    fn nested_triple_as_subject() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        {
            ta.push_triple_start();
            ta.try_push_subject(|b| bn(b, "a"))?;
            ta.try_push_predicate(|b| iri(b, "b"))?;
            ta.try_push_object(|b, _| sl(b, "c"))?;
            assert_eq!(format!("{}", ta.top()), r#"_:a <b> "c""#);
        }
        ta.push_subject_triple();
        ta.try_push_predicate(|b| iri(b, "d"))?;
        ta.try_push_object(|b, _| sl(b, "e"))?;
        assert_eq!(format!("{}", ta.top()), r#"<< _:a <b> "c" >> <d> "e""#);
        Ok(())
    }

    #[test]
    fn nested_triple_as_object() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        ta.try_push_subject(|b| bn(b, "a"))?;
        ta.try_push_predicate(|b| iri(b, "b"))?;
        {
            ta.push_triple_start();
            ta.try_push_subject(|b| bn(b, "c"))?;
            ta.try_push_predicate(|b| iri(b, "d"))?;
            ta.try_push_object(|b, _| sl(b, "e"))?;
            assert_eq!(format!("{}", ta.top()), r#"_:c <d> "e""#);
        }
        ta.push_object_triple();
        assert_eq!(format!("{}", ta.top()), r#"_:a <b> << _:c <d> "e" >>"#);
        Ok(())
    }

    #[test]
    fn nested_triple_as_both() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        {
            ta.push_triple_start();
            ta.try_push_subject(|b| bn(b, "a"))?;
            ta.try_push_predicate(|b| iri(b, "b"))?;
            ta.try_push_object(|b, _| sl(b, "c"))?;
            assert_eq!(format!("{}", ta.top()), r#"_:a <b> "c""#);
        }
        ta.push_subject_triple();
        ta.try_push_predicate(|b| iri(b, "d"))?;
        {
            ta.push_triple_start();
            ta.try_push_subject(|b| bn(b, "e"))?;
            ta.try_push_predicate(|b| iri(b, "f"))?;
            ta.try_push_object(|b, _| sl(b, "g"))?;
            assert_eq!(format!("{}", ta.top()), r#"_:e <f> "g""#);
        }
        ta.push_object_triple();
        assert_eq!(
            format!("{}", ta.top()),
            r#"<< _:a <b> "c" >> <d> << _:e <f> "g" >>"#
        );
        Ok(())
    }

    #[test]
    fn nested_triple_deep() -> Result<(), Infallible> {
        let mut ta = TripleAllocator::_new();
        ta.push_triple_start();
        {
            ta.push_triple_start();
            ta.try_push_subject(|b| bn(b, "a"))?;
            ta.try_push_predicate(|b| iri(b, "b"))?;
            {
                ta.push_triple_start();
                ta.try_push_subject(|b| bn(b, "c"))?;
                ta.try_push_predicate(|b| iri(b, "d"))?;
                ta.try_push_object(|b, _| sl(b, "e"))?;
                assert_eq!(format!("{}", ta.top()), r#"_:c <d> "e""#);
            }
            ta.push_object_triple();
            assert_eq!(format!("{}", ta.top()), r#"_:a <b> << _:c <d> "e" >>"#);
        }
        ta.push_subject_triple();
        ta.try_push_predicate(|b| iri(b, "f"))?;
        {
            ta.push_triple_start();
            {
                ta.push_triple_start();
                ta.try_push_subject(|b| bn(b, "g"))?;
                ta.try_push_predicate(|b| iri(b, "h"))?;
                ta.try_push_object(|b, _| sl(b, "i"))?;
                assert_eq!(format!("{}", ta.top()), r#"_:g <h> "i""#);
            }
            ta.push_subject_triple();
            ta.try_push_predicate(|b| iri(b, "j"))?;
            ta.try_push_object(|b, _| sl(b, "k"))?;
            assert_eq!(format!("{}", ta.top()), r#"<< _:g <h> "i" >> <j> "k""#);
        }
        ta.push_object_triple();
        assert_eq!(
            format!("{}", ta.top()),
            r#"<< _:a <b> << _:c <d> "e" >> >> <f> << << _:g <h> "i" >> <j> "k" >>"#
        );

        ta.pop_top_triple();
        assert_eq!(ta.complete_len, 0);
        assert_eq!(ta.incomplete_len, 0);
        Ok(())
    }
}
