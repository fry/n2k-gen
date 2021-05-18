use canutils::candump_parser::*;
use embedded_hal_can::{Filter, Frame};
use n2k_messages::Pgns;
use std::convert::TryFrom;

struct CanDumpReceiver {
    lines: Vec<String>,
    ctr: usize,
}

impl CanDumpReceiver {
    pub fn new(dump_file: &str) -> Self {
        let dump = std::fs::read_to_string(dump_file).unwrap();
        Self {
            lines: dump.lines().map(|s| s.to_owned()).collect(),
            ctr: 0,
        }
    }
}
impl embedded_hal_can::Interface for CanDumpReceiver {
    type Id = n2k::Id;
    type Frame = n2k::CanFrame;

    type Error = ();
    type Filter = MockFilter;
}
struct MockFilter {}

impl Filter for MockFilter {
    type Id = n2k::Id;

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

impl embedded_hal_can::Receiver for CanDumpReceiver {
    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        loop {
            if self.ctr >= self.lines.len() {
                return Err(nb::Error::WouldBlock);
            }
            let entry = dump_entry(&self.lines[self.ctr]);
            self.ctr += 1;

            if let Ok(entry) = entry {
                let id = n2k::Id::try_from(entry.1.can_frame().frame_id).unwrap();
                let bytes = entry.1.can_frame().frame_body.to_be_bytes();
                return Ok(n2k::CanFrame::new(id, &bytes));
            }
        }
    }

    fn set_filter(&mut self, filter: Self::Filter) {
        panic!();
    }

    fn clear_filter(&mut self) {
        panic!();
    }
}

fn main() {
    env_logger::init();
    let receiver = CanDumpReceiver::new("candumpSample3.txt");
    let mut bus: n2k::Bus<_, n2k_messages::PgnRegistry> = n2k::Bus::new(receiver);

    loop {
        let result = bus.receive();
        if !matches!(result, Ok(None)) {
            dbg!(result);
        }
    }
}
