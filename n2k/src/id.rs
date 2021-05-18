use crate::GLOBAL_ADDRESS;
use core::convert::TryFrom;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdError {
    CanNotSendToDestination,
    DestinationRequired,
    InvalidId,
    InvalidPriority,
}

pub type Result<T> = core::result::Result<T, IdError>;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Priority {
    Priority0 = 0,
    Priority1 = 1,
    Priority2 = 2,
    Priority3 = 3,
    Priority4 = 4,
    Priority5 = 5,
    Priority6 = 6,
    Priority7 = 7,
}

#[derive(Clone, Copy)]
pub struct Id(u32);

impl Id {
    //TODO: figure out if this should be split into two constructors
    pub fn new(prio: Priority, pgn: u32, src: u8, dst: u8) -> Result<Self> {
        let mut id: u32 = 0x00;

        id |= src as u32 & 0xff;

        let pf = (pgn >> 8) & 0xff;
        if pf <= 239 {
            // PDU 1
            id |= (dst as u32 & 0xff) << 8;
            id |= pgn << 8;
        } else {
            if dst != GLOBAL_ADDRESS {
                return Err(IdError::CanNotSendToDestination);
            }
            // PDU 2
            id |= pgn << 8;
        }
        id |= (prio as u32) << 26;

        Ok(Id(id))
    }

    pub fn priority(&self) -> Priority {
        let prio: u8 = ((self.0 >> 26) & 0x7) as u8;
        match prio {
            0 => Priority::Priority0,
            1 => Priority::Priority1,
            2 => Priority::Priority2,
            3 => Priority::Priority3,
            4 => Priority::Priority4,
            5 => Priority::Priority5,
            6 => Priority::Priority6,
            7 => Priority::Priority7,
            _ => panic!("Invalid priority"),
        }
    }

    pub fn pgn(&self) -> u32 {
        let pf: u8 = (self.0 >> 16) as u8;
        let dp: u8 = ((self.0 >> 24) & 1) as u8;
        if pf <= 239 {
            // PDU1 format, the PS contains the destination address
            let pgn = ((dp as u32) << 16) + ((pf as u32) << 8);
            pgn as u32
        } else {
            // PDU2 format, the PGN is extended
            let ps: u8 = (self.0 >> 8) as u8;
            let pgn = ((dp as u32) << 16) + ((pf as u32) << 8) + (ps as u32);
            pgn as u32
        }
    }

    pub fn source(&self) -> u8 {
        self.0 as u8
    }

    pub fn destination(&self) -> u8 {
        let pf: u8 = (self.0 >> 16) as u8;
        if pf <= 239 {
            // PDU1 format, the PS contains the destination address
            let ps: u8 = (self.0 >> 8) as u8;
            ps
        } else {
            // PDU2 format, the destination is implied global and the PGN is extended
            0xff
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl core::fmt::Debug for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Id")
            .field("priority", &self.priority())
            .field("pgn", &self.pgn())
            .field("source", &self.source())
            .field("destination", &self.destination())
            .finish()
    }
}

impl crate::hal_can::Id for Id {
    type BaseId = ();

    type ExtendedId = u32;

    fn base_id(&self) -> Option<Self::BaseId> {
        None
    }

    fn extended_id(&self) -> Option<Self::ExtendedId> {
        Some(self.value())
    }
}

impl TryFrom<u32> for Id {
    type Error = IdError;

    fn try_from(val: u32) -> Result<Id> {
        validate_id(&val)?;

        Ok(Id(val))
    }
}

fn validate_id(id: &u32) -> Result<()> {
    if id & 0xe0000000 > 0 {
        return Err(IdError::InvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Id, Priority, GLOBAL_ADDRESS};
    use core::convert::TryFrom;

    #[test]
    fn id_new_destination() {
        struct TestCase {
            id: u32,
            prio: Priority,
            pgn: u32,
            src: u8,
            dst: u8,
        }
        let test_cases = [
            TestCase {
                id: 0x18eafc00,
                prio: Priority::Priority6,
                pgn: 59904,
                src: 0,
                dst: 252,
            },
            TestCase {
                id: 0x1cecff3d,
                prio: Priority::Priority7,
                pgn: 60416,
                src: 61,
                dst: GLOBAL_ADDRESS,
            },
            TestCase {
                id: 0xcfe6cee,
                prio: Priority::Priority3,
                pgn: 65132,
                src: 238,
                dst: 255,
            },
        ];
        for i in &test_cases {
            let id: u32 = Id::new(i.prio, i.pgn, i.src, i.dst)
                .expect("Invalid parameter")
                .value();
            assert_eq!(id, i.0)
        }
    }

    #[test]
    fn id_priority() {
        struct TestCase {
            id: u32,
            prio: Priority,
        }
        let test_cases = [
            TestCase {
                id: 0x18eafc00,
                prio: Priority::Priority6,
            },
            TestCase {
                id: 0x1cecff3d,
                prio: Priority::Priority7,
            },
            TestCase {
                id: 0xcfe6cee,
                prio: Priority::Priority3,
            },
        ];
        for i in &test_cases {
            let id = Id::try_from(i.0).expect("Invalid CanID");
            assert_eq!(id.priority(), i.prio)
        }
    }

    #[test]
    fn id_pgn() {
        struct TestCase {
            id: u32,
            pgn: u32,
        }
        let test_cases = [
            TestCase {
                id: 0x18eafc00,
                pgn: 59904,
            },
            TestCase {
                id: 0x1cecff3d,
                pgn: 60416,
            },
            TestCase {
                id: 0xcfe6cee,
                pgn: 65132,
            },
        ];
        for i in &test_cases {
            let id = Id::try_from(i.0).expect("Invalid CanID");
            assert_eq!(id.pgn(), i.pgn)
        }
    }

    #[test]
    fn id_source() {
        struct TestCase {
            id: u32,
            src: u8,
        }
        let test_cases = [
            TestCase {
                id: 0x18eafc00,
                src: 0,
            },
            TestCase {
                id: 0x1cecff3d,
                src: 61,
            },
            TestCase {
                id: 0xcfe6cee,
                src: 238,
            },
        ];
        for i in &test_cases {
            let id = Id::try_from(i.0).expect("Invalid CanID");
            assert_eq!(id.source(), i.src)
        }
    }

    #[test]
    fn id_destination() {
        struct TestCase {
            id: u32,
            dst: u8,
        }
        let test_cases = [
            TestCase {
                id: 0x18eafc00,
                dst: 252,
            },
            TestCase {
                id: 0x1cecff3d,
                dst: 255,
            },
            TestCase {
                id: 0xcfe6cee,
                dst: 255,
            },
        ];
        for i in &test_cases {
            let id = Id::try_from(i.0).expect("Invalid CanID");
            assert_eq!(id.destination(), i.dst)
        }
    }
}
