use std::{collections::HashMap, fs::File, iter, sync::Arc};

use eframe::egui::mutex::Mutex;
use indexmap::IndexMap;
use io_bluetooth::bt::{self, BtStream};
use native_dialog::FileDialog;

use crate::{MyApp, ADDRESS, NUM_PEDALS};

const CONNECTION_ATTEMPTS: usize = 3;

const ALIASES: [(&str, &str); 4] = [
    ("Down", "CC52"),
    ("Up", "CC53"),
    ("Tune", "CC68"),
    ("Tap", "CC64"),
];

pub struct IConnection {
    pub socket: Option<BtStream>,
    pub status: String,
    pub new_connect: bool,
}

pub type Connection = Arc<Mutex<IConnection>>;

impl Default for IConnection {
    fn default() -> Self {
        Self {
            socket: Option::None,
            status: "Not connected".to_string(),
            new_connect: false,
        }
    }
}

pub fn get_connection() -> Connection {
    Arc::new(Mutex::new(IConnection::default()))
}

pub fn try_connect(connection: Connection) {
    connection.lock().status = "Connecting".to_string();

    for i in 0..CONNECTION_ATTEMPTS {
        let mut devices = match bt::discover_devices() {
            Ok(d) => d,
            Err(_) => {
                connection.lock().status = "Bluetooth not available!".to_string();
                return;
            }
        };

        devices = devices
            .into_iter()
            .filter(|d| *d.to_string() == ADDRESS.to_string())
            .collect();

        if devices.len() == 1 {
            let socket =
                BtStream::connect(iter::once(&devices[0]), bt::BtProtocol::RFCOMM).unwrap();

            match socket.peer_addr() {
                Ok(_) => {}
                Err(err) => println!("An error occured while retrieving the peername: {:?}", err),
            }

            match socket.local_addr() {
                Ok(_) => {}
                Err(err) => println!("An error occured while retrieving the sockname: {:?}", err),
            }

            connection.lock().socket = Option::from(socket);
            connection.lock().status = "Connected".to_string();
            connection.lock().new_connect = true;
            return;
        }

        // sleep(Duration::from_millis(1500));
        connection.lock().status = format!("{}. Attempt failed", i + 1);
    }

    connection.lock().status = "Connection failed".to_string();
}

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
        self.connection.lock().status.clone()
    }

    pub fn req_cfg(&mut self) {
        let cfg_req: Vec<u8> = vec![0x00];

        let connection = self.connection.clone();
        let s = &connection.lock().socket;

        let socket = match s {
            None => return,
            Some(s) => s,
        };
        socket.send(&cfg_req).unwrap();

        let mut cfg_buffer: Vec<u8> = vec![0; (2 * NUM_PEDALS + 1) as usize];

        socket.recv(&mut cfg_buffer).unwrap();

        cfg_buffer.pop();

        let mut index = 0;

        let mut loaded_config = self.columns.lock().unwrap();

        loop {
            let ped = cfg_buffer[index];
            let value = self.cfg_str_from_value(cfg_buffer[index + 1]);

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

    fn cfg_str_from_value(&self, value: u8) -> String {
        let mut type_st = "".to_string();
        let mut input = value;

        let msg_type = input & 0x80;
        match msg_type {
            0 => type_st += "PC",
            128 => type_st += "CC",
            _ => {}
        }

        input = input & 0x7F;
        type_st += &(input as i32).to_string();

        match self.match_alias_rev(&type_st) {
            Some(s) => type_st = s.clone(),
            _ => {}
        }

        type_st
    }

    fn command_from_str(&self, cmd: &String) -> Option<u8> {
        let mut num_value: u8;

        let input: String;

        match self.match_alias(&cmd) {
            Some(n) => input = n.clone(),
            None => input = cmd.clone(),
        }

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

        let path;

        match input {
            None => return,
            Some(p) => path = p,
        }

        let file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let mut cfg: std::collections::HashMap<u8, String> = HashMap::new();
        self.columns.lock().unwrap().iter().for_each(|(k, v)| {
            cfg.insert(*k, v.clone());
        });
        // cfg.insert(0xFF, self.tempo_list.clone());
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
            Some(p) => path = p,
        }

        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}", why),
            Ok(file) => file,
        };

        let cfg: std::collections::HashMap<u8, String> = serde_json::from_reader(file).unwrap();
        let mut res = IndexMap::new();

        // self.tempo_list = cfg.remove(&0xFF).unwrap();
        cfg.iter().for_each(|(k, v)| {
            res.insert(*k, v.clone());
        });

        *self.columns.lock().unwrap() = res;

        self.sort_cfg();
    }

    pub fn send_cfg(&mut self) {
        let mut output_buffer: Vec<u8> = Vec::with_capacity((NUM_PEDALS * 2 + 1) as usize);
        output_buffer.push(0x01);

        let cfg = self.columns.lock().unwrap();

        for (pedal, input) in cfg.iter() {
            let num_value = self.command_from_str(input).unwrap();

            output_buffer.push(*pedal);
            output_buffer.push(num_value);
        }
        output_buffer.push(0x00);

        self.connection
            .lock()
            .socket
            .as_ref()
            .unwrap()
            .send(&output_buffer)
            .unwrap();
        drop(cfg);
    }

    // pub fn _send_midi_command(&mut self) {
    //     let mut output_buffer: Vec<u8> = Vec::with_capacity((NUM_PEDALS * 2 + 1) as usize);
    //     output_buffer.push(0x02);

    //     let command = self.command_from_str(&self.custom_cmd).unwrap();

    //     output_buffer.push(command);

    //     self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    // }

    // pub fn _send_pedal_command(&mut self) {
    //     let mut output_buffer: Vec<u8> = Vec::with_capacity((NUM_PEDALS * 2 + 1) as usize);
    //     output_buffer.push(0x03);

    //     output_buffer.push(self.custom_pedal_nr.parse().unwrap());

    //     self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    // }

    // pub fn _send_tempo_change(&mut self) {
    //     let mut output_buffer: Vec<u8> = Vec::with_capacity((NUM_PEDALS * 2 + 1) as usize);
    //     output_buffer.push(0x04);

    //     output_buffer.append(&mut Self::tempo_bytes_from_str(&self.tempo));

    //     self.socket.as_ref().unwrap().send(&output_buffer).unwrap();
    // }

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

    fn tempo_bytes_from_str(input: &str) -> Vec<u8> {
        let f_value: f32 = input.parse().unwrap();
        let mut res = vec![];
        f_value.to_le_bytes().iter().for_each(|b| res.push(*b));

        res
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
