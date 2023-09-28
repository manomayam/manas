//! I define bare minimal ntriples serializer
//! that serializes in async streaming way natively.
//!

use async_stream::try_stream;
use manas_http::representation::impl_::common::data::{
    bytes_stream::BoxBytesStream, quads_stream::BoxQuadsStream,
};
use sophia_api::quad::Quad;
use sophia_turtle::serializer::nt;

pub fn serialize_default_graph_to_ntriples(quads: BoxQuadsStream) -> BoxBytesStream {
    Box::pin(try_stream! {
        for await quad_result in quads {
            let quad = quad_result?;
            let mut buf = Vec::new();
            for term in quad.spog().into_triple().into_iter() {
                nt::write_term(&mut buf, term)?;
                buf.push(b' ');
            }
            buf.extend(b".\n");
            yield buf.into();
        }
    })
}
