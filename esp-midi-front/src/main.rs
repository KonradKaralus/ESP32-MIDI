use std::collections::HashMap;
use std::path::PathBuf;

use native_dialog::FileDialog;

use std::io;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;

use io_bluetooth::bt::{self, BtStream};

use slint::SharedString;

const NUM_PEDALS:u8 = 5; 

const TEST:bool = true; 


mod cfg_utils;

use cfg_utils::*;

fn main() -> Result<(), std::io::Error> {

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

    let start = FileDialog::new()
    .set_location("~/Documents")
    .add_filter(".json", &["json"])
    .show_open_single_file()
    .unwrap();

    

    let mut path = PathBuf::new();
    let load;

    match start {
        Some(p) => {path = p; load = true;},
        None => {load=false;}
    }

    if load {
        println!("path{:?}", path);
        todo!();
    } else {
        req_cfg(&socket, &mut loaded_config.lock().unwrap());
    }

    let app = App::new().unwrap();

    let mut_app = Arc::new(app);
    let app_for_lambda = mut_app.clone();
    let app_for_lambda2 = mut_app.clone();


    let cl = move |ped:i32, val:SharedString| {       
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
        
    };


    mut_app.on_txtchng(cl);
    mut_app.on_subm_clicked(submit);

    mut_app.on_namechng(namechng);


    update_current_cfg(&mut loaded_config.lock().unwrap(), Option::None, *&mut_app.as_ref());

    mut_app.run().unwrap();



    // let mut buffer = vec![0; 1024];
    // loop {
    //     match socket.recv(&mut buffer[..]) {
    //         Ok(len) => println!("Received {} bytes.", len),
    //         Err(err) => return Err(err),
    //     }
    // }

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