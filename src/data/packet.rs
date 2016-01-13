//! Generate data packets.

use bits;
use data::{self, payload, coder, interleave};

/// Construct a confirmed data packet with the given header, payload blocks, and serial
/// number generator. The returned value is the coded, interleaved set of dibit symbols
/// that make up the packet.
pub fn confirmed<S>(header: data::ConfirmedHeader, payload: data::ConfirmedPayload,
                    mut sn: S)
    -> Vec<bits::Dibit> where S: Iterator<Item = u8>
{
    let mut pkt = vec![];

    // Add in the header.
    pkt.extend({
        let (fields, checksum) = header.build();

        coder::DibitCoder::new()
            .feed_bytes(fields.iter().cloned())
            .feed_bytes(checksum.iter().cloned())
            .finish()
            .iter().cloned()
    });

    // Add in the normal data blocks.
    for block in payload.iter() {
        pkt.extend({
            let (data, pads) = block.build();
            let header = payload::ConfirmedBlockHeader::new(sn.next().unwrap(),
                data, pads.clone()).build();

            interleave::Interleaver::new(coder::TribitCoder::new()
                .feed_bytes(header.iter().cloned()
                    .chain(data.iter().cloned())
                    .chain(pads.map(|_| 0)))
                .finish())
        });
    }

    // Add in the tail block.
    pkt.extend({
        let (data, pads, checksum) = payload.tail().build();
        let header = payload::ConfirmedBlockHeader::new(sn.next().unwrap(),
            data, pads.clone()).build();

        interleave::Interleaver::new(coder::TribitCoder::new()
            .feed_bytes(header.iter().cloned()
                .chain(data.iter().cloned())
                .chain(pads.map(|_| 0))
                .chain(checksum.iter().cloned()))
            .finish())
    });

    pkt
}

/// Construct an unconfirmed data packet with the given header and payload blocks. The
/// returned value is the coded, interleaved set of dibit symbols that make up the packet.
pub fn unconfirmed(header: data::UnconfirmedHeader, payload: data::UnconfirmedPayload)
    -> Vec<bits::Dibit>
{
    let mut pkt = vec![];

    pkt.extend({
        let (fields, checksum) = header.build();

        coder::DibitCoder::new()
            .feed_bytes(fields.iter().cloned())
            .feed_bytes(checksum.iter().cloned())
            .finish()
            .iter().cloned()
    });

    for block in payload.iter() {
        let (data, pads) = block.build();

        pkt.extend({
            interleave::Interleaver::new(coder::DibitCoder::new()
                .feed_bytes(data.iter().cloned())
                .feed_bytes(pads.map(|_| 0))
                .finish())
        });
    }

    pkt.extend({
        let (data, pads, checksum) = payload.tail().build();

        interleave::Interleaver::new(coder::DibitCoder::new()
            .feed_bytes(data.iter().cloned())
            .feed_bytes(pads.map(|_| 0))
            .feed_bytes(checksum.iter().cloned())
            .finish())
    });

    pkt
}
