use crate::Id;

/// A CAN data or remote frame.
#[derive(Clone, Debug)]
pub struct CanFrame {
    id: Id,
    dlc: usize,
    data: [u8; 8],
}

impl CanFrame {
    /// Creates a new data frame.
    pub fn new(id: Id, data: &[u8]) -> Self {
        let mut frame = Self {
            id,
            dlc: data.len(),
            data: [0; 8],
        };
        frame.data[0..data.len()].copy_from_slice(data);
        frame
    }
}

impl crate::hal_can::Frame for CanFrame {
    type Id = Id;

    /// Returns true if this frame is a remote frame
    fn is_remote_frame(&self) -> bool {
        false
    }
    /// Returns true if this frame is a data frame
    fn is_data_frame(&self) -> bool {
        !self.is_remote_frame()
    }
    /// Returns the frame data (0..8 bytes in length).
    fn data(&self) -> Option<&[u8]> {
        if self.is_data_frame() {
            Some(&self.data[0..self.dlc])
        } else {
            None
        }
    }

    fn id(&self) -> Self::Id {
        self.id
    }
}
