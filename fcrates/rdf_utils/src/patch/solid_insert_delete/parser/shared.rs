use std::{char, io::BufRead, u8};

use oxilangtag::LanguageTag;
use oxiri::Iri;
use rio_api::model::*;

use super::{error::*, utils::*};

pub const MAX_ASCII: u8 = 0x7F;

pub fn parse_iriref_absolute<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
) -> Result<NamedNode<'a>, TurtleError> {
    parse_iriref(read, buffer)?;
    Iri::parse(buffer.as_str()).map_err(|error| {
        read.parse_error(TurtleErrorKind::InvalidIri {
            iri: buffer.to_owned(),
            error,
        })
    })?;
    Ok(NamedNode { iri: buffer })
}

pub fn parse_iriref_relative<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
    temp_buffer: &mut String,
    base_iri: &Option<Iri<String>>,
) -> Result<NamedNode<'a>, TurtleError> {
    if let Some(base_iri) = base_iri {
        parse_iriref(read, temp_buffer)?;
        let result = base_iri.resolve_into(temp_buffer, buffer).map_err(|error| {
            read.parse_error(TurtleErrorKind::InvalidIri {
                iri: temp_buffer.to_owned(),
                error,
            })
        });
        temp_buffer.clear();
        result.map(move |_| NamedNode { iri: buffer })
    } else {
        parse_iriref_absolute(read, buffer)
    }
}

pub fn parse_iriref(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [18] 	IRIREF 	::= 	'<' ([^#x00-#x20<>"{}|^`\] | UCHAR)* '>' /* #x00=NULL #01-#x1F=control codes #x20=space */
    // Most of the validation is done by the IRI parser
    read.check_is_current(b'<')?;
    loop {
        read.consume()?;
        match read.current() {
            None | Some(b'\n') | Some(b'\r') => read.unexpected_char_error()?,
            Some(b'>') => {
                read.consume()?;
                return Ok(());
            }
            Some(b'\\') => {
                read.consume()?;
                buffer.push(match read.current() {
                    Some(b'u') => read_hexa_char(read, 4)?,
                    Some(b'U') => read_hexa_char(read, 8)?,
                    _ => read.unexpected_char_error()?,
                });
            }
            Some(c) => buffer.push(if c <= MAX_ASCII {
                char::from(c) //optimization to avoid UTF-8 decoding
            } else {
                read_utf8_char(read)?
            }),
        }
    }
}

pub fn parse_blank_node_label<'a>(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &'a mut String,
) -> Result<BlankNode<'a>, TurtleError> {
    // [141s] 	BLANK_NODE_LABEL 	::= 	'_:' (PN_CHARS_U | [0-9]) ((PN_CHARS | '.')* PN_CHARS)?
    read.check_is_current(b'_')?;
    read.consume()?;
    read.check_is_current(b':')?;
    read.consume()?;

    let c = read.required_current()?;
    if c <= MAX_ASCII && (is_possible_pn_chars_u_ascii(c) || (b'0'..=b'9').contains(&c)) {
        buffer.push(char::from(c))
    } else {
        let c = read_utf8_char(read)?;
        if is_possible_pn_chars_u_unicode(c) {
            buffer.push(c);
        } else {
            read.unexpected_char_error()?
        }
    }

    loop {
        read.consume()?;
        match read.current() {
            Some(b'.') => match read.next()? {
                Some(c) if is_possible_pn_chars_ascii(c) || c > MAX_ASCII => buffer.push('.'),
                _ => {
                    return Ok(BlankNode { id: buffer });
                }
            },
            Some(c) if c < MAX_ASCII && is_possible_pn_chars_ascii(c) => buffer.push(char::from(c)),
            _ => {
                let c = read_utf8_char(read)?;
                if is_possible_pn_chars_unicode(c) {
                    buffer.push(c);
                } else {
                    return Ok(BlankNode { id: buffer });
                }
            }
        }
    }
}

pub fn parse_langtag(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [144s] 	LANGTAG 	::= 	'@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
    read.check_is_current(b'@')?;
    read.consume()?;

    while let Some(c) = read.current() {
        match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' => {
                buffer.push(char::from(c).to_ascii_lowercase());
                read.consume()?;
            }
            _ => {
                break;
            }
        }
    }
    LanguageTag::parse(buffer.as_str()).map_err(|error| {
        read.parse_error(TurtleErrorKind::InvalidLanguageTag {
            tag: buffer.to_owned(),
            error,
        })
    })?;
    Ok(())
}

pub fn parse_string_literal_quote(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    // [22] 	STRING_LITERAL_QUOTE 	::= 	'"' ([^#x22#x5C#xA#xD] | ECHAR | UCHAR)* '"' /* #x22=" #x5C=\ #xA=new line #xD=carriage return */
    parse_string_literal_quote_inner(read, buffer, b'"')
}

pub fn parse_string_literal_quote_inner(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
    quote: u8,
) -> Result<(), TurtleError> {
    read.check_is_current(quote)?;
    loop {
        read.consume()?;
        match read.required_current()? {
            c if c == quote => {
                read.consume()?;
                return Ok(());
            }
            b'\\' => parse_echar_or_uchar(read, buffer)?,
            b'\n' | b'\r' => read.unexpected_char_error()?,
            c => buffer.push(if c <= MAX_ASCII {
                char::from(c) //optimization to avoid UTF-8 decoding
            } else {
                read_utf8_char(read)?
            }),
        }
    }
}

pub fn parse_echar_or_uchar(
    read: &mut LookAheadByteReader<impl BufRead>,
    buffer: &mut String,
) -> Result<(), TurtleError> {
    read.check_is_current(b'\\')?;
    read.consume()?;
    match read.required_current()? {
        b't' => buffer.push('\t'),
        b'b' => buffer.push('\u{8}'),
        b'n' => buffer.push('\n'),
        b'r' => buffer.push('\r'),
        b'f' => buffer.push('\u{C}'),
        b'"' => buffer.push('"'),
        b'\'' => buffer.push('\''),
        b'\\' => buffer.push('\\'),
        b'u' => buffer.push(read_hexa_char(read, 4)?),
        b'U' => buffer.push(read_hexa_char(read, 8)?),
        _ => read.unexpected_char_error()?,
    }
    Ok(())
}

pub(crate) fn read_hexa_char(
    read: &mut LookAheadByteReader<impl BufRead>,
    len: usize,
) -> Result<char, TurtleError> {
    let point = read_hexa_u32(read, len)?;
    char::from_u32(point)
        .ok_or_else(|| read.parse_error(TurtleErrorKind::InvalidUnicodeCodePoint(point)))
}

fn read_hexa_u32(
    read: &mut LookAheadByteReader<impl BufRead>,
    len: usize,
) -> Result<u32, TurtleError> {
    let mut value = 0;
    for _ in 0..len {
        read.consume()?;
        if let Some(d) = convert_hexa_byte(read.required_current()?) {
            value = value * 16 + u32::from(d);
        } else {
            read.unexpected_char_error()?;
        };
    }
    Ok(value)
}

fn convert_hexa_byte(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// [157s] 	PN_CHARS_BASE 	::= 	[A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
pub fn is_possible_pn_chars_base_ascii(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z')
}

// [157s] 	PN_CHARS_BASE 	::= 	[A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
pub fn is_possible_pn_chars_base_unicode(c: char) -> bool {
    matches!(c,
        'A'..='Z'
        | 'a'..='z'
        | '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}'
        | '\u{00F8}'..='\u{02FF}'
        | '\u{0370}'..='\u{037D}'
        | '\u{037F}'..='\u{1FFF}'
        | '\u{200C}'..='\u{200D}'
        | '\u{2070}'..='\u{218F}'
        | '\u{2C00}'..='\u{2FEF}'
        | '\u{3001}'..='\u{D7FF}'
        | '\u{F900}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFFD}'
        | '\u{10000}'..='\u{EFFFF}')
}

// [158s] 	PN_CHARS_U 	::= 	PN_CHARS_BASE | '_' | ':'
pub fn is_possible_pn_chars_u_ascii(c: u8) -> bool {
    is_possible_pn_chars_base_ascii(c) || c == b'_'
}

// [158s] 	PN_CHARS_U 	::= 	PN_CHARS_BASE | '_' | ':'
pub fn is_possible_pn_chars_u_unicode(c: char) -> bool {
    is_possible_pn_chars_base_unicode(c) || c == '_'
}

// [160s] 	PN_CHARS 	::= 	PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
pub fn is_possible_pn_chars_ascii(c: u8) -> bool {
    is_possible_pn_chars_u_ascii(c) || matches!(c, b'-' | b'0'..=b'9' | 0x00B7)
}

// [160s] 	PN_CHARS 	::= 	PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
pub fn is_possible_pn_chars_unicode(c: char) -> bool {
    is_possible_pn_chars_u_unicode(c)
        || matches!(c,
        '-' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
}

/// Algorithm from https://encoding.spec.whatwg.org/#utf-8-decoder
pub fn read_utf8_char(read: &mut LookAheadByteReader<impl BufRead>) -> Result<char, TurtleError> {
    let mut code_point: u32;
    let bytes_needed: usize;
    let mut lower_boundary = 0x80;
    let mut upper_boundary = 0xBF;

    let byte = read.required_current()?;
    match byte {
        0x00..=0x7F => return Ok(char::from(byte)),
        0xC2..=0xDF => {
            bytes_needed = 1;
            code_point = u32::from(byte) & 0x1F;
        }
        0xE0..=0xEF => {
            if byte == 0xE0 {
                lower_boundary = 0xA0;
            }
            if byte == 0xED {
                upper_boundary = 0x9F;
            }
            bytes_needed = 2;
            code_point = u32::from(byte) & 0xF;
        }
        0xF0..=0xF4 => {
            if byte == 0xF0 {
                lower_boundary = 0x90;
            }
            if byte == 0xF4 {
                upper_boundary = 0x8F;
            }
            bytes_needed = 3;
            code_point = u32::from(byte) & 0x7;
        }
        _ => return read.unexpected_char_error(),
    }

    for _ in 0..bytes_needed {
        read.consume()?;
        let byte = read.required_current()?;
        if byte < lower_boundary || upper_boundary < byte {
            return read.unexpected_char_error();
        }
        lower_boundary = 0x80;
        upper_boundary = 0xBF;
        code_point = (code_point << 6) | (u32::from(byte) & 0x3F);
    }

    char::from_u32(code_point)
        .ok_or_else(|| read.parse_error(TurtleErrorKind::InvalidUnicodeCodePoint(code_point)))
}
