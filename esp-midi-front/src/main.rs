use core::time;
use std::collections::HashMap;
use std::rc::Rc;

use std::io;
use std::iter;

use io_bluetooth::bt::{self, BtStream};

use slint::{ModelRc, SharedString, VecModel};

const NUM_PEDALS:u8 = 5; 

const TEST:bool = true; 

slint::slint!{
    import { Button, VerticalBox, HorizontalBox, LineEdit} from "std-widgets.slint";

    export component App inherits Window {
        preferred-width: 800px;
        preferred-height: 600px;

        in property <[int]> names;

        callback txtchng(int, string);
        
        VerticalBox {
            for it in names: HorizontalBox {
                Text {
                    height: 30px;
                    text:it;
                }
                LineEdit {
                    height: 30px;
                    placeholder-text:"Enter";
                    accepted(string) => {txtchng(it, string);}
                }
            }
        }
    }
}

fn cfg_str_from_value(value:u8) -> String {

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

fn main() -> Result<(), std::io::Error> {

    let mut loaded_config: HashMap<u8,String> = HashMap::with_capacity(NUM_PEDALS as usize); 

    let ped_vec:Vec<i32> = (0..NUM_PEDALS as i32).collect();
       
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

    let cfg_req:Vec<u8> = vec![0x00];

    socket.send(&cfg_req).unwrap();

    std::thread::sleep(time::Duration::from_secs(1));

    let mut cfg_buffer:Vec<u8> = vec![0; (2*NUM_PEDALS + 1) as usize];

    socket.recv(&mut cfg_buffer).unwrap();

    cfg_buffer.pop();

    let mut index = 0;

    loop {

        let ped = cfg_buffer[index];
        let value = cfg_str_from_value(cfg_buffer[index]);

        loaded_config.insert(ped, value);
        index += 2;
        if index as u8 > NUM_PEDALS {
            break;
        }
    }



    println!("vec: {:?}", loaded_config);

    let cl = move |ped:i32, val:SharedString| {       
        let input = val.as_str().to_string();
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

        let buf:[u8;2] = [ped as u8, num_value];

        if TEST {
            println!("{:?}, {:?}", buf[0], buf[1]);
            return;
        }

        socket.send(&buf).unwrap();
    };

    let the_model : Rc<VecModel<i32>> =
        Rc::new(VecModel::from(Vec::from(ped_vec)));
    let the_model_rc = ModelRc::from(the_model.clone());

    let app =App::new().unwrap();
    app.set_names(the_model_rc);

    app.on_txtchng(cl);

    // app.run().unwrap();



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