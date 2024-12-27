use bytemuck::{Pod, Zeroable};

use crate::CC_SEP;

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Command {
    /// 0: PC, 255: CC
    signal_type: u8,
    /// 0-127
    value: u8,
    /// 0-127
    on_activate: u8,
    /// 0-127
    on_deactivate: u8,
}

impl Command {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn new_pc(value: u8) -> Self {
        assert!(value < 128);
        Self {
            signal_type: 0,
            value,
            on_activate: 0xff,
            on_deactivate: 0xff,
        }
    }

    pub fn new_cc_simple(value: u8) -> Self {
        assert!(value < 128);
        Self {
            signal_type: 0xff,
            value,
            on_activate: 0xff,
            on_deactivate: 0xff,
        }
    }

    pub fn new_cc(value: u8, on_activate: u8, on_deactivate: u8) -> Self {
        assert!(value < 128 && on_activate < 128 && on_deactivate < 128);
        Self {
            signal_type: 0xff,
            value,
            on_activate,
            on_deactivate,
        }
    }

    pub fn type_str(&self) -> &str {
        match self.signal_type {
            0 => "PC",
            255 => "CC",
            _ => "Err", //TODO: Option wrap
        }
    }

    pub fn value_str(&self) -> String {
        self.value.to_string()
    }

    pub fn option_str(&self) -> String {
        match self.on_activate {
            0xff => "".to_string(),
            _ => format!("{}{},{}", CC_SEP, self.on_activate, self.on_deactivate),
        }
    }
}
