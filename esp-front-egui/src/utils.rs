use std::{collections::HashMap, fs::File};

use indexmap::IndexMap;
use native_dialog::FileDialog;

use crate::{MyApp, NUM_PEDALS};

impl MyApp {
    pub fn _print_current_cfg(&mut self) {
        println!("current cfg: {:?}", *self.columns.lock().unwrap());
    }

    pub fn _cfg_str(&self) -> String {
        format!("current cfg: {:?}", *self.columns.lock().unwrap())
    }
    
    
    pub fn req_cfg(&mut self) {
        let cfg_req:Vec<u8> = vec![0x00];

        let socket = match &self.socket {
            None => return,
            Some(s) => s
        };
    
        socket.send(&cfg_req).unwrap();
        
        let mut cfg_buffer:Vec<u8> = vec![0; (2*NUM_PEDALS + 1) as usize];
    
        socket.recv(&mut cfg_buffer).unwrap();
    
        cfg_buffer.pop();
    
        let mut index = 0;

        let mut loaded_config = self.columns.lock().unwrap();
    
        loop {
            let ped = cfg_buffer[index];
            let value = Self::cfg_str_from_value(cfg_buffer[index+1]);
    
            if value.is_empty() {
                panic!("was not CC or PC")
            }

            match self.match_alias(value.clone()) {
                Some(s) => loaded_config.insert(ped, s),
                None => loaded_config.insert(ped, value)
            };
    
            
            index += 2;
            if index as u8 >= NUM_PEDALS*2 {
                break;
            }
        }
        drop(loaded_config);
        self.sort_cfg();
    }
    
    
    fn cfg_str_from_value(value:u8) -> String {
    
        let mut type_st = "".to_string();
        let mut input = value;
    
        let msg_type = input & 0x80;
        match msg_type {
            0 => type_st += "PC",
            1 => type_st += "CC",
            _ => {}
        }  
    
        input = input & 0x7F;  
        type_st += &(input as i32).to_string();  
    
        type_st
    }

    fn command_from_str(&self, cmd:&String) -> Option<u8> {
            let mut num_value:u8;

            let input:String;

            match self.match_alias(cmd.clone()) {
                Some(n) => input = n,
                None => input = cmd.clone()
            }
            
            if input.contains("CC") {
                let value = input.replace("CC", "");
                num_value = value.parse::<u8>().unwrap();
                num_value += 128; //set first bit
            }
            else if input.contains("PC") {
                let value = input.replace("PC", "");
                num_value = value.parse::<u8>().unwrap();  
            }
            else {
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
    
        let path;
    
        match input {
            None => return,
            Some(p) => path = p
        }
    
        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let mut cfg:std::collections::HashMap<u8,String> = HashMap::new();
        self.columns.lock().unwrap().iter().for_each(|(k,v)| {
            cfg.insert(*k, v.clone());
        });
        cfg.insert(0xFF, self.tempo_list.clone());
        serde_json::to_writer(file, &cfg).unwrap();
    }
    
    pub fn load_cfg(&mut self) {
        let input = FileDialog::new()
        .set_location("~/Documents")
        .add_filter(".json", &["json"])
        .show_open_single_file()
        .unwrap();
    
        let path;
    
        match input {
            None => return,
            Some(p) => path = p
        }
    
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };
    
        let mut cfg:std::collections::HashMap<u8, String> = serde_json::from_reader(file).unwrap();
        let mut res = IndexMap::new();

        self.tempo_list = cfg.remove(&0xFF).unwrap();
        cfg.iter().for_each(|(k,v)| {
            res.insert(*k,v.clone());
        });

        *self.columns.lock().unwrap() = res;

        self.sort_cfg();
    }

    pub fn send_cfg(&mut self) {

        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0x01);

        let cfg = self.columns.lock().unwrap();
    
        for (pedal, input) in cfg.iter() {
            let num_value = self.command_from_str(input).unwrap();
    
            output_buffer.push(*pedal);
            output_buffer.push(num_value);
        }
        output_buffer.push(0x00);

        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();

        drop(cfg);
    }

    pub fn send_midi_command(&mut self) {
        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0x02);

        let command = self.command_from_str(&self.custom_cmd).unwrap();

        output_buffer.push(command);

        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    }

    pub fn send_pedal_command(&mut self) {
        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0x03);

        output_buffer.push(self.custom_pedal_nr.parse().unwrap());

        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    }

    pub fn send_tempo_change(&mut self) {
        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0x04);

        output_buffer.append(&mut Self::tempo_bytes_from_str(&self.tempo));

        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    }

    pub fn send_tempo_list(&mut self) {
        let mut output_buffer:Vec<u8> = Vec::new();
        output_buffer.push(0x05);

        let tempos:Vec<&str> = self.tempo_list.split(",").collect();

        for tempo in tempos {
            output_buffer.append(&mut Self::tempo_bytes_from_str(tempo));
        }
        for _ in 0..5 {
            output_buffer.push(0x00);
        }
        println!("sending tempolist: {:?}", output_buffer);
        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    }

    fn sort_cfg(&mut self) {
        let mut new_cfg:IndexMap<u8, String> = IndexMap::new();

        let mut collect:Vec<(u8, String)> = vec![];

        self.columns.lock().unwrap().iter().for_each(|(k,v)| {
            collect.push((*k,v.clone()))
        });

        collect.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());

        collect.iter().for_each(|(k,v)| {
            new_cfg.insert(*k,v.clone());
        });   

        *self.columns.lock().unwrap() = new_cfg;
    }

    fn match_alias(&self, input:String) -> Option<String> {

        for (a,b) in &self.aliases {
            if *a == input {
                return Option::from(b.clone());
            }
            if *b == input {
                return Option::from(a.clone());
            }
        }
        return Option::None;
    }

    fn tempo_bytes_from_str(input:&str) -> Vec<u8> {
        let f_value:f32 = input.parse().unwrap();
        let mut res = vec![];
        f_value.to_le_bytes().iter().for_each(|b| res.push(*b));

        res
    }

    pub fn get_aliases() -> HashMap<String, String> {
        let mut res = HashMap::new();
        res.insert("down".into(), "CC52".into());
        res.insert("up".into(), "CC53".into());
        res.insert("tun".into(), "CC68".into());
        res.insert("tnext".into(), "CC127".into());
        res.insert("T".into(), "CC64".into());

        res
    }
}