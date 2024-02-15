use core::time;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

use native_dialog::FileDialog;
use slint::Model;
use slint::ModelRc;
use slint::VecModel;

use std::io;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;
use io_bluetooth::bt::{self, BtStream};

use slint::SharedString;

const NUM_PEDALS:u8 = 5; 

const TEST:bool = true; 

slint::include_modules!();



fn main() -> Result<(), std::io::Error> {

    let mut patch_name = "".to_string();

    let loaded_config_map: HashMap<u8,String> = HashMap::with_capacity(NUM_PEDALS as usize); 

    let loaded_config = Arc::new(Mutex::new(loaded_config_map));
    let loaded_config_lambda_cl = loaded_config.clone();
    let loaded_config_lambda_submit = loaded_config.clone();
      
    let devices = bt::discover_devices()?;
    println!("Devices:");
    for (idx, device) in devices.iter().enumerate() {
        println!("{}: {}", idx, *device);
    }

    if devices.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No Bluetooth devices found.",
        ));
    }

    let device_idx = request_device_idx(devices.len())?;

    let socket = BtStream::connect(iter::once(&devices[device_idx]), bt::BtProtocol::RFCOMM)?;

    match socket.peer_addr() {
        Ok(_) => {},
        Err(err) => println!("An error occured while retrieving the peername: {:?}", err),
    }

    match socket.local_addr() {
        Ok(_) => {},
        Err(err) => println!("An error occured while retrieving the sockname: {:?}", err),
    }

    req_cfg(&socket, &mut loaded_config.lock().unwrap());


    let app = App::new().unwrap();

    let mut_app = Arc::new(app);
    let app_for_lambda = mut_app.clone();
    // let app_for_lambda2 = mut_app.clone();

    let change_value = move |ped:i32, val:SharedString| {       
        update_current_cfg(&mut loaded_config_lambda_cl.lock().unwrap(), Option::Some((ped,val)), &app_for_lambda);

        if TEST {
            print_current_cfg(&loaded_config_lambda_cl.lock().unwrap());
            return;
        }
    };

    let submit = move || {
        send_cfg(&loaded_config_lambda_submit.lock().unwrap(), &socket);
    };

    let namechng = move |name:SharedString| {
        patch_name = name.into();
    };

    let save = move || {};
    let load = move || {};
    let get = move || {};



    mut_app.on_subm_clicked(submit);

    mut_app.on_txtchng(change_value);

    mut_app.on_namechng(namechng);

    mut_app.on_save_clicked(save);
    mut_app.on_load_clicked(load);
    mut_app.on_get_clicked(get);

    update_current_cfg(&mut loaded_config.lock().unwrap(), Option::None, &mut_app);

    mut_app.run().unwrap();
    Ok(())
}


fn request_device_idx(len: usize) -> io::Result<usize> {
    println!("Please specify the index of the Bluetooth device you want to connect to:");

    let mut buffer = String::new();
    loop {
        io::stdin().read_line(&mut buffer)?;
        if let Ok(idx) = buffer.trim_end().parse::<usize>() {
            if idx < len {
                return Ok(idx);
            }
        }
        buffer.clear();
        println!("Invalid index. Please try again.");
    }
}

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

    serde_json::to_writer(file, cfg).unwrap();
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


fn print_lines(app:&App) {
    let l = app.get_lines();
    let iter =l.iter();

    for line in iter {
        println!("{:?}; {:?}", line.name, line.value);
    }
}