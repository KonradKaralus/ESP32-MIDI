use std::{cmp::min, collections::HashMap, fs::File};

use indexmap::IndexMap;
use native_dialog::FileDialog;

use crate::{MyApp, NUM_PEDALS, TEST};

impl MyApp {
    pub fn print_current_cfg(&mut self) {
        // println!("current cfg: {:?}", *self.columns.lock().unwrap());
        self.console(format!("Current cfg: {}", self.cfg_str()));
    }

    pub fn cfg_str(&self) -> String {
        format!("current cfg: {:?}", *self.columns.lock().unwrap())
    }
    
    
    pub fn req_cfg(&mut self) {
        let cfg_req:Vec<u8> = vec![0x00];

        let socket = match &self.socket {
            None => return,
            Some(s) => s
        };
    
        socket.send(&cfg_req).unwrap();
    
        // std::thread::sleep(time::Duration::from_secs(1));
    
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
    
            loaded_config.insert(ped, value);
            index += 2;
            if index as u8 >= NUM_PEDALS*2 {
                break;
            }
        }
        if TEST {
            println!("loaded cfg: {:?}", loaded_config);
        }
        drop(loaded_config);
        self.sort_cfg();

        self.console(format!("Received cfg: {}", self.cfg_str()));
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

    fn command_from_str(cmd:&String) -> Option<u8> {
            let mut num_value:u8;

            let input:String;

            match Self::match_alias(cmd.clone()) {
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
    
        serde_json::to_writer(file, &cfg).unwrap();

        self.console(format!("Saved cfg"));
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
    
        let cfg:std::collections::HashMap<u8, String> = serde_json::from_reader(file).unwrap();

        self.console(format!("Loaded cfg: {:?}", cfg));

        let mut res = IndexMap::new();
        
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
            let num_value = Self::command_from_str(input).unwrap();
    
            output_buffer.push(*pedal);
            output_buffer.push(num_value);
        }

        if TEST {
            println!("sending {:?}", output_buffer);
        }
        self.socket.as_ref().unwrap().send(&output_buffer).unwrap();

        drop(cfg);

        self.console(format!("Sent cfg: {}", self.cfg_str()));
    }

    pub fn send_midi_command(&mut self) {
        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0x02);

        let command = Self::command_from_str(&self.custom_cmd).unwrap();

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

        let f_value:f32 = self.tempo.parse().unwrap();

        f_value.to_le_bytes().iter().for_each(|b| output_buffer.push(*b));

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

    pub fn console(&mut self, s:String) {
        self.console.push(s);
    }

    pub fn get_last_line(&self) -> String {
        let con = &self.console;

        let l = con.len();
        let lower = min(4, l);

        let mut out = vec![];

        for idx in l-lower..l {
            out.push(con[idx].clone());
        }
        
        out.join("\n")
    }   

    fn match_alias(input:String) -> Option<String> {
        match input.as_str() {
            "down" => Option::from("CC52".to_string()),
            "up" =>  Option::from("CC53".to_string()),
            "tun" => Option::from("CC68".to_string()),
            "snext" => Option::from("CC127".to_string()),
            _ => Option::None
        }
    }
}