use std::{collections::HashMap, fs::File};

use indexmap::IndexMap;
use native_dialog::FileDialog;

use crate::{MyApp, NUM_PEDALS, TEST};

impl MyApp {
    pub fn print_current_cfg(&self) {
        println!("current cfg: {:?}", *self.columns.lock().unwrap());
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

    pub fn serialize_cfg(&self) {

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

        let mut res = IndexMap::new();

        cfg.iter().for_each(|(k,v)| {
            res.insert(*k,v.clone());
        });
        
        *self.columns.lock().unwrap() = res;
    }

    pub fn send_cfg(&self) {

        let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
        output_buffer.push(0xFF);

        let cfg = self.columns.lock().unwrap();
    
        for (pedal, input) in cfg.iter() {
            let mut num_value:u8 = 0;
    
            if input.contains("CC") {
                let value = input.replace("CC", "");
                num_value = value.parse::<u8>().unwrap();
                num_value += 128; //set first bit
            }
            else if input.contains("PC") {
                let value = input.replace("PC", "");
                num_value = value.parse::<u8>().unwrap();  
            }
    
            output_buffer.push(*pedal);
            output_buffer.push(num_value);
            }
            if TEST {
                println!("sending {:?}", output_buffer);
            }
            self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    }
    
}

