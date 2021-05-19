use core::{convert::TryFrom, fmt::Debug, marker::PhantomData};

use heapless::FnvIndexMap;

use crate::{
    fast_packet,
    hal_can::{self, Receiver, Transmitter},
    message::MessageError,
};
use crate::{CanFrame, PgnRegistry};
use crate::{Id, IdError, Message, GLOBAL_ADDRESS};

const CB_TP_BAM: u8 = 0x40; // Control byte indicating TP_BAM

const PGN_TP_CM: u32 = 0x00ec00; // 60416 - ISO Transport Protocol, Connection Management - RTS group
const PGN_TP_DT: u32 = 0x00eb00; // 60160 - ISO Transport Protocol, Data Transfer

#[derive(Copy, Clone, Debug)]
pub enum BusError<E, P> {
    CouldNotOpenBus,
    CouldNotSendMessage,
    NoExtendedId,
    NoData,
    InvalidId(IdError),
    MessageError(MessageError),
    OutOfFastPacketMemory,
    FastPacket(fast_packet::FastPacketError),
    CanError(E),
    PgnError(P),
}

impl<E, P> From<IdError> for BusError<E, P> {
    fn from(error: IdError) -> Self {
        BusError::InvalidId(error)
    }
}

impl<E, P> From<MessageError> for BusError<E, P> {
    fn from(error: MessageError) -> Self {
        BusError::MessageError(error)
    }
}

impl<E, P> From<fast_packet::FastPacketError> for BusError<E, P> {
    fn from(error: fast_packet::FastPacketError) -> Self {
        BusError::FastPacket(error)
    }
}
// impl<E> From<E> for BusError<E> {
//     fn from(error: MessageError) -> Self {
//         BusError::CanError(error)
//     }
// }

pub type Result<T, E, P> = core::result::Result<T, BusError<E, P>>;

pub struct Bus<T, P> {
    can: T,
    address: u8,
    // fast packet assembly cache, size must be a power of two, not currently enforced at compile time by FnvIndexMap
    fast_packet_cache:
        FnvIndexMap<fast_packet::FastPacketIdentifier, fast_packet::FastPacketCache, 16>,
    _pgn_registry: PhantomData<P>,
}

impl<T, P> Bus<T, P> {
    pub fn new(can: T) -> Self {
        Bus {
            can,
            address: 0,
            fast_packet_cache: FnvIndexMap::new(),
            _pgn_registry: PhantomData,
        }
    }
}

impl<T, E, I, F, P> Bus<T, P>
where
    E: core::fmt::Debug,
    I: hal_can::Id<ExtendedId = u32>,
    F: hal_can::Frame<Id = I>,
    T: Receiver<Frame = F, Error = E>,
    P: PgnRegistry,
{
    pub fn receive(&mut self) -> nb::Result<Option<P::Message>, BusError<E, P::Error>> {
        // Consume at most one frame without blocking, propagate errors
        let frame = match self.can.receive() {
            Ok(frame) => frame,
            Err(nb::Error::WouldBlock) => return Ok(None),
            Err(nb::Error::Other(e)) => return Err(nb::Error::Other(BusError::CanError(e))),
        };

        // NMEA2000 only uses extended IDs
        if frame.id().extended_id().is_none() {
            return Err(BusError::NoExtendedId.into());
        }

        let id = Id::try_from(frame.id().extended_id().unwrap()).map_err(BusError::InvalidId)?;
        let data = if let Some(data) = frame.data() {
            data
        } else {
            return Err(BusError::NoData.into());
        };
        // Is fast packet?
        if P::is_fast_packet(id.pgn()) {
            // Good explanation of the fast packet bit format:
            // https://forums.ni.com/t5/LabVIEW/How-do-I-read-the-larger-than-8-byte-messages-from-a-NMEA-2000/td-p/3132045?profile.language=en

            let fp_seq_nr = data[0] & 0xE0;
            let fp_index = (data[0] & 0x1F) as usize;

            log::info!("received fast packet PGN {}, index {}", id.pgn(), fp_index);
            // Identifier for the particular fast packet
            let message_id = (id.source(), id.pgn(), fp_seq_nr);

            // First fast packet frame, initialize cache
            if fp_index == 0 {
                let fp_data = &data[2..];
                let total_size = data[1] as usize;
                log::info!("total size {}", total_size);

                let mut cache = fast_packet::FastPacketCache::new(total_size);
                let result = cache.extend(fp_index, fp_data);
                if result.is_err() {
                    self.fast_packet_cache.remove(&message_id);
                }
                self.fast_packet_cache
                    .insert(message_id, cache)
                    .map_err(|_| BusError::OutOfFastPacketMemory)?;
                log::info!("fast packet initialized as {:?}", message_id);
            } else {
                // Subsequent packet
                let fp_data = &data[1..];
                if let Some(cache) = self.fast_packet_cache.get_mut(&message_id) {
                    let result = cache.extend(fp_index, fp_data);
                    log::info!("fast packet data {}/{}", cache.data.len(), cache.total_size);

                    if result.is_err() {
                        // Error extending packet, remove cache
                        self.fast_packet_cache.remove(&message_id);
                    } else if let Some(data) = cache.complete_data() {
                        // Packet is complete
                        let message =
                            P::build_message(id.pgn(), data).map_err(BusError::PgnError)?;
                        self.fast_packet_cache.remove(&message_id);
                        return Ok(Some(message));
                    }
                } else {
                    log::error!(
                        "received invalid frame index {} for unknown fast packet {:?}",
                        fp_index,
                        message_id
                    );
                }
            }

            // Nothing complete yet
            Ok(None)
        } else {
            // Simple single-frame message
            let message = P::build_message(id.pgn(), data).map_err(BusError::PgnError)?;
            Ok(Some(message))
        }
    }
}

impl<T, E, P> Bus<T, P>
where
    E: core::fmt::Debug,
    T: Transmitter<Frame = CanFrame, Error = E>,
    P: PgnRegistry,
{
    pub fn send(&mut self, message: &Message) -> Result<(), E, P::Error> {
        let id = message.id();
        let data = message.data();
        let length = data.len();

        if length <= 8 {
            //TODO: Make sure it's not a fast packet
            let frame = CanFrame::new(id, data);
            self.transmit(&frame)?;
            Ok(())
        } else {
            // Send a broadcast ISO 11783 multi-packet
            //calculate number of packets that will be sent
            let packets = (length / 7) + 1;
            // send broadcast announce message (BAM)
            let pgn = id.pgn();
            let priority = id.priority();
            let tp_cm_id = Id::new(priority, PGN_TP_CM, self.address, GLOBAL_ADDRESS)?;
            let tp_cm_id_data = [
                CB_TP_BAM,                    // Control Byte: TP_BAM
                (length & 0xff) as u8,        // message size LSB
                ((length >> 8) & 0xff) as u8, // message size MSB
                packets as u8,                // number of packets
                0xff,                         // maximun number of packets
                (pgn & 0xff) as u8,           // PGN LSB
                ((pgn >> 8) & 0xff) as u8,    // PGN
                ((pgn >> 16) & 0xff) as u8,   // PGN MSB
            ];

            let frame = CanFrame::new(tp_cm_id, &tp_cm_id_data);
            self.transmit(&frame)?;

            // send packets
            let tp_dt_id = Id::new(priority, PGN_TP_DT, self.address, GLOBAL_ADDRESS)?;
            let mut count = 1;
            let mut index = 0;
            let mut remaining = length;
            let mut len;
            while remaining > 0 {
                len = remaining;
                if len > 7 {
                    len = 7;
                }
                remaining -= len;

                // fill data
                let mut tp_dt_data = [255; 8];

                tp_dt_data[0] = count;
                count += 1;
                for i in 0..len {
                    tp_dt_data[i + 1] = data[index];
                    index += 1;
                }

                let frame = CanFrame::new(tp_dt_id, &tp_dt_data);
                self.transmit(&frame)?;
            }

            Ok(())
        }
    }

    fn transmit(&mut self, frame: &CanFrame) -> Result<(), E, P::Error> {
        // TODO: revise this as it's not looking optimal or correct
        let result = self.can.transmit(frame);
        match result {
            Ok(None) => Ok(()),
            // A lower priority frame was replaced with our high priority frame.
            // Put the low priority frame back in the transmit queue.
            Ok(pending_frame) => {
                if let Some(f) = pending_frame {
                    self.transmit(&f)
                } else {
                    Ok(())
                }
            }
            Err(nb::Error::WouldBlock) => self.transmit(frame), // Need to retry
            Err(nb::Error::Other(e)) => Err(BusError::CanError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use crate::hal_can::{Filter, Frame, Interface, Receiver, Transmitter};
    use crate::{Bus, Id, Message, Priority, GLOBAL_ADDRESS};

    use crate::frame::*;
    struct MockCan {
        pub frames: Vec<CanFrame>,
    }

    impl MockCan {
        pub fn new() -> Self {
            MockCan { frames: Vec::new() }
        }
    }

    struct MockFilter {}

    impl Filter for MockFilter {
        type Id = Id;

        fn from_id(_id: Self::Id) -> Self {
            panic!();
        }

        fn accept_all() -> Self {
            panic!();
        }

        fn from_mask(_mask: u32, _filter: u32) -> Self {
            panic!();
        }
    }

    impl Interface for MockCan {
        type Frame = CanFrame;
        type Id = Id;
        type Error = ();
        type Filter = MockFilter;
    }

    impl Receiver for MockCan {
        fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
            panic!();
        }

        fn set_filter(&mut self, _filter: Self::Filter) {
            panic!();
        }

        fn clear_filter(&mut self) {
            panic!();
        }
    }

    impl Transmitter for MockCan {
        fn transmit(&mut self, frame: &CanFrame) -> nb::Result<Option<Self::Frame>, Self::Error> {
            self.frames.push(frame.clone());
            Ok(Option::None)
        }
    }

    #[test]
    fn bus_send() {
        struct TestCase {
            message: Message,
        }
        let test_cases = [
            TestCase {
                message: Message::new(
                    Id::new(Priority::Priority0, 12345, 123, GLOBAL_ADDRESS).unwrap(),
                    Box::new([1, 2, 3, 4, 5, 6, 7]),
                )
                .unwrap(),
            },
            TestCase {
                message: Message::new(
                    Id::new(Priority::Priority0, 12345, 123, GLOBAL_ADDRESS).unwrap(),
                    Box::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]),
                )
                .unwrap(),
            },
        ];
        for i in &test_cases {
            let can = MockCan::new();
            let mut bus = Bus::new(can);

            bus.send(&i.message).unwrap();

            let data = i.message.data();
            if data.len() <= 8 {
                // Single packet
            } else {
                // Multipacket
                for b in 0..data.len() {
                    let frame = (b / 7) + 1;
                    let index = b - ((frame - 1) * 7) + 1;
                    assert_eq!(bus.can.frames[frame].data().unwrap()[index], data[b])
                }
            }
        }
    }
}
