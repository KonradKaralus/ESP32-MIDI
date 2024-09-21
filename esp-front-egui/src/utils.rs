use std::{collections::HashMap, fs::File};

use indexmap::IndexMap;
use native_dialog::FileDialog;

use crate::{MyApp, NUM_PEDALS};

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
        let mut cfg = o_cfg.unwrap();
        cfg.pop();

        let mut index = 0;

        let mut loaded_config = self.columns.lock().unwrap();

        loop {
            let ped = cfg[index];
            let value = self.cfg_str_from_value(cfg[index + 1]);

            if value.is_empty() {
                panic!("was not CC or PC")
            }

            loaded_config.insert(ped, value);

            index += 2;
            if index as u8 >= NUM_PEDALS * 2 {
                break;
            }
        }
        drop(loaded_config);
        self.sort_cfg();
    }

    pub fn send_cfg(&mut self) {
        let mut output_buffer: Vec<u8> = Vec::with_capacity((NUM_PEDALS * 2 + 1) as usize);

        let cfg = self.columns.lock().unwrap();

        for (pedal, input) in cfg.iter() {
            let num_value = self.command_from_str(input).unwrap();

            output_buffer.push(*pedal);
            output_buffer.push(num_value);
        }
        output_buffer.push(0x00);

        self.connection.lock().send_cfg(output_buffer);
    }

    fn cfg_str_from_value(&self, value: u8) -> String {
        let mut type_st = "".to_string();
        let mut input = value;

        let msg_type = input & 0x80;
        match msg_type {
            0 => type_st += "PC",
            128 => type_st += "CC",
            _ => {}
        }

        input &= 0x7F;
        type_st += &(input as i32).to_string();

        if let Some(s) = self.match_alias_rev(&type_st) {
            type_st = s.clone()
        }

        type_st
    }

    fn command_from_str(&self, cmd: &String) -> Option<u8> {
        let mut num_value: u8;

        let input: String = match self.match_alias(cmd) {
            Some(n) => n.clone(),
            None => cmd.clone(),
        };

        if input.contains("CC") {
            let value = input.replace("CC", "");
            num_value = value.parse::<u8>().unwrap();
            num_value += 128; //set first bit
        } else if input.contains("PC") {
            let value = input.replace("PC", "");
            num_value = value.parse::<u8>().unwrap();
        } else {
            return Option::None;
        }

        Option::from(num_value)
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
