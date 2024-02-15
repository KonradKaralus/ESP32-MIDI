use core::time;
use std::{collections::HashMap, rc::Rc};

use io_bluetooth::bt::BtStream;
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

const NUM_PEDALS:u8 = 5; 
const TEST:bool = true;


pub fn cfg_str_from_value(value:u8) -> String {

    let mut type_st = "".to_string();
    let mut input = value;

    let msg_type = input & 0x80;
    match msg_type {
        0 => type_st += "PC",
        1 => type_st += "CC",
        _ => println!("invalid msg type")
    }  

    input = input & 0x7F;  
    type_st += &(input as i32).to_string();  

    type_st
}

pub fn serialize_cfg(cfg:&HashMap<u8,String>) {
    //let se = serde_json::to_writer(writer, cfg);
    //serde_json::from_reader(rdr)
    todo!();
}


pub fn update_current_cfg(cfg:&mut HashMap<u8,String>, new:Option<(i32,SharedString)>, app:&App) {

    match new {
        Some(n) => {cfg.insert(n.0 as u8, n.1.to_string());},
        None => {}
    }

    let mut line_vec = vec![];

    for (name, value) in cfg {
        let l = Line {
            name:*name as i32,
            value:SharedString::from(value.clone())
        };
        line_vec.push(l);
    }

    line_vec.sort_by(|a,b| a.name.partial_cmp(&b.name).unwrap());

    let the_model : Rc<VecModel<Line>> =
        Rc::new(VecModel::from(line_vec));
    let the_model_rc = ModelRc::from(the_model.clone());

    app.set_lines(the_model_rc);
}

pub fn print_current_cfg(cfg:&HashMap<u8,String>) {
    println!("current cfg: {:?}", cfg);
}

pub fn send_cfg(cfg:&HashMap<u8,String>, socket:&BtStream) {

    let mut output_buffer:Vec<u8> = Vec::with_capacity((NUM_PEDALS*2 + 1) as usize);
    output_buffer.push(0xFF);

    for (pedal, input) in cfg {
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

        socket.send(&output_buffer).unwrap();
}


pub fn req_cfg(socket:&BtStream, loaded_config:&mut HashMap<u8,String>) {
    let cfg_req:Vec<u8> = vec![0x00];

    socket.send(&cfg_req).unwrap();

    std::thread::sleep(time::Duration::from_secs(1));

    let mut cfg_buffer:Vec<u8> = vec![0; (2*NUM_PEDALS + 1) as usize];

    socket.recv(&mut cfg_buffer).unwrap();

    cfg_buffer.pop();

    let mut index = 0;

    loop {
        let ped = cfg_buffer[index];
        let value = cfg_str_from_value(cfg_buffer[index+1]);

        loaded_config.insert(ped, value);
        index += 2;
        if index as u8 >= NUM_PEDALS*2 {
            break;
        }
    }
    println!("loaded cfg: {:?}", loaded_config);

}
