use bytemuck::{Pod, Zeroable};

use crate::COMMAND_SEP;

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Command {
    /// 0: PC, 255: CC
    signal_type: u8,
    /// 0-127
    value: u8,
    /// 0-127, if this is 0xff, then on_activate and on_deactivate will have the same value
    on_activate: u8,
    /// 0-127
    on_deactivate: u8,
    /// MIDI-channel
    channel: u8,
}

impl Command {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn as_str(&self) -> Option<String> {
        let mut res = "".to_string();

        res += match self.signal_type {
            0 => "PC",
            255 => "CC",
            _ => {
                return None;
            }
        };

        res += &self.value.to_string();

        let options = match self.on_activate {
            0xff => "",
            _ => &format!("{}{},{}", COMMAND_SEP, self.on_activate, self.on_deactivate),
        };

        res += options;

        let channel = match self.channel {
            0x01 => "",
            0x00..0x0f => &format!("{}c{}", COMMAND_SEP, self.channel),
            _ => {
                return None;
            }
        };

        res += channel;

        Some(res)
    }

    /// PC<num> or CC<num>|<ac>,<deac>|c<channel> (channel and ac,dc are optional)
    pub fn from_string(mut input: String) -> Option<Self> {
        let mut res = Self::default();

        // parse type
        if input.contains("CC") {
            input = input.replace("CC", "");
            res.signal_type = 0xff;
        } else if input.contains("PC") {
            input = input.replace("PC", "");
            res.signal_type = 0x00;
        } else {
            return None;
        }

        let split: Vec<&str> = input.split(COMMAND_SEP).collect();

        // parse Command Value
        if split[0].chars().any(|c| !c.is_numeric()) || split[0].len() > 3 || split[0].is_empty() {
            return None;
        }
        res.value = split[0].parse().unwrap();

        // parse options
        for v in &split[1..split.len()] {
            // channel
            if v.starts_with("c") {
                let stripped = v.replace("c", "");
                let v_o = stripped.parse();
                if v_o.is_err() {
                    return None;
                }
                res.channel = v_o.unwrap();
            }
            // ac, deac TODO: does this need it's own identifier?
            else {
                let opts_o: Vec<Result<u8, _>> = v.split(",").map(|n| n.parse::<u8>()).collect();
                if opts_o.iter().any(|n| n.is_err()) || opts_o.len() != 2 {
                    return None;
                }
                res.on_activate = *opts_o[0].as_ref().unwrap();
                res.on_deactivate = *opts_o[1].as_ref().unwrap();
            }
        }

        Some(res)
    }
}

impl Default for Command {
    fn default() -> Self {
        Self {
            signal_type: 0x00,
            value: 0x00,
            on_activate: 0xff,
            on_deactivate: 0xff,
            channel: 0x01,
        }
    }
}
