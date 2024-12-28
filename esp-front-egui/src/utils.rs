use std::{collections::HashMap, fs::File};

use indexmap::IndexMap;
use native_dialog::FileDialog;

use crate::{command::Command, MyApp, CC_SEP, NUM_PEDALS};

const ALIASES: [(&str, &str); 4] = [
    ("Down", "CC52"),
    ("Up", "CC53"),
    ("Tune", "CC68"),
    ("Tap", "CC64"),
];

impl MyApp {
    pub fn _print_current_cfg(&mut self) {
        println!("current cfg: {:?}", *self.columns.lock().unwrap());
    }

    pub fn is_connected(&self) -> bool {
        self.connection.lock().socket.is_some()
    }

    pub fn is_newly_connected(&self) -> bool {
        if self.connection.lock().new_connect {
            self.connection.lock().new_connect = false;
            return true;
        }
        false
    }

    pub fn _cfg_str(&self) -> String {
        format!("current cfg: {:?}", *self.columns.lock().unwrap())
    }

    pub fn get_connection_status(&self) -> String {
        self.connection.lock().status.fmt()
    }

    pub fn req_cfg(&mut self) {
        let o_cfg = self.connection.lock().req_cfg();
        if o_cfg.is_none() {
            return;
        }
        let cfg = o_cfg.unwrap();
        // cfg.pop();

        let mut index = 0;

        let mut loaded_config = self.columns.lock().unwrap();

        loop {
            let ped = cfg[index];
            let cmd: Command = *bytemuck::from_bytes(&cfg[index + 1..index + size_of::<Command>()]);
            let value = self.cfg_str_from_value(cmd);

            if value.is_empty() {
                panic!("was not CC or PC")
            }

            loaded_config.insert(ped, value);

            index += size_of::<Command>() + 1;
            if index >= cfg.len() {
                break;
            }
        }
        drop(loaded_config);
        self.sort_cfg();
    }

    pub fn send_cfg(&mut self) {
        let signal_size = size_of::<Command>();

        let mut output_buffer: Vec<u8> =
            Vec::with_capacity(NUM_PEDALS + NUM_PEDALS * signal_size + 1);
        let cfg = self.columns.lock().unwrap();

        for (pedal, input) in cfg.iter() {
            let num_value = self.command_from_str(input).unwrap();

            output_buffer.push(*pedal);
            output_buffer.extend_from_slice(num_value.as_bytes());
        }
        drop(cfg);

        self.connection.lock().send_cfg(output_buffer);
    }

    fn cfg_str_from_value(&self, value: Command) -> String {
        let mut type_st = "".to_string();
        let input = value;

        type_st += input.type_str();

        // input &= 0x7F;
        // type_st += &(input as i32).to_string();

        type_st += &input.value_str();
        type_st += &input.option_str();

        if let Some(s) = self.match_alias_rev(&type_st) {
            type_st = s.clone()
        }

        type_st
    }

    // PC<num> or CC<num>|<ac>,<deac>
    fn command_from_str(&self, cmd: &String) -> Option<Command> {
        let input: String = match self.match_alias(cmd) {
            Some(n) => n.clone(),
            None => cmd.clone(),
        };

        if input.contains("CC") {
            let stripped = input.replace("CC", "");

            if !stripped.contains(CC_SEP) {
                let o_value = stripped.parse();
                if o_value.is_err() {
                    return None;
                }
                let value: u8 = o_value.unwrap();

                return Some(Command::new_cc_simple(value));
            }

            let split: Vec<&str> = stripped.split("|").collect();
            if split.len() != 2 {
                return None;
            }

            let o_value = split[0].parse();
            if o_value.is_err() {
                return None;
            }
            let value: u8 = o_value.unwrap();

            let o_opts: Vec<Result<u8, _>> = split[1].split(",").map(|n| n.parse::<u8>()).collect();
            if o_opts.iter().any(|n| n.is_err()) || o_opts.len() != 2 {
                return None;
            }
            let on_activate = *o_opts[0].as_ref().unwrap();
            let on_deactivate = *o_opts[1].as_ref().unwrap();

            return Some(Command::new_cc(value, on_activate, on_deactivate));
        } else if input.contains("PC") {
            let o_value = input.replace("PC", "").parse();
            if o_value.is_err() {
                return None;
            }
            let value: u8 = o_value.unwrap();

            return Some(Command::new_pc(value));
        }

        None
    }

    pub fn serialize_cfg(&mut self) {
        let input = FileDialog::new()
            .set_location("~/Documents")
            .add_filter(".json", &["json"])
            .show_save_single_file()
            .unwrap();

        let path = match input {
            None => return,
            Some(p) => p,
        };

        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let mut cfg: std::collections::HashMap<u8, String> = HashMap::new();
        self.columns.lock().unwrap().iter().for_each(|(k, v)| {
            cfg.insert(*k, v.clone());
        });
        serde_json::to_writer(file, &cfg).unwrap();
    }

    pub fn load_cfg(&mut self) {
        let input = FileDialog::new()
            .set_location("~/Documents")
            .add_filter(".json", &["json"])
            .show_open_single_file()
            .unwrap();

        let path = match input {
            None => return,
            Some(p) => p,
        };

        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let cfg: std::collections::HashMap<u8, String> = serde_json::from_reader(file).unwrap();
        let mut res = IndexMap::new();

        cfg.iter().for_each(|(k, v)| {
            res.insert(*k, v.clone());
        });

        *self.columns.lock().unwrap() = res;

        self.sort_cfg();
    }

    fn sort_cfg(&mut self) {
        let mut new_cfg: IndexMap<u8, String> = IndexMap::new();

        let mut collect: Vec<(u8, String)> = vec![];

        self.columns
            .lock()
            .unwrap()
            .iter()
            .for_each(|(k, v)| collect.push((*k, v.clone())));

        collect.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        collect.iter().for_each(|(k, v)| {
            new_cfg.insert(*k, v.clone());
        });

        *self.columns.lock().unwrap() = new_cfg;
    }

    fn match_alias(&self, input: &String) -> Option<&String> {
        self.aliases.get(input)
    }

    fn match_alias_rev(&self, input: &String) -> Option<&String> {
        self.aliases_rev.get(input)
    }

    pub fn get_aliases() -> (HashMap<String, String>, HashMap<String, String>) {
        let mut res = HashMap::new();
        let mut res2 = HashMap::new();

        for (key, val) in ALIASES {
            res.insert(key.to_string(), val.to_string());
            res2.insert(val.to_string(), key.to_string());
        }

        (res, res2)
    }
}
