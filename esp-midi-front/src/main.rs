use core::time;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use native_dialog::FileDialog;

use std::io;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;

use io_bluetooth::bt::{self, BtStream};

use slint::{ModelRc, SharedString, VecModel};

const NUM_PEDALS:u8 = 5; 

const TEST:bool = true; 

slint::slint!{
    import { Button, VerticalBox, HorizontalBox, LineEdit, GroupBox, StandardListView, ListView} from "std-widgets.slint";

    export struct Line  {
            name: int,
            value: string
        }

    export component App inherits Window {
        preferred-width: 800px;
        preferred-height: 600px;

        in property <[Line]> lines;
        in property <string> name;

        callback txtchng(int, string);
        callback subm_clicked <=> submit.clicked;
        
        callback namechng(string);

        VerticalBox {

        HorizontalBox {
            submit := Button { 
                text: "Send";
                height: 50px;
            }

            save := Button {
                text: "Save";
                height: 50px;
            }

            get := Button {
                text: "Get from device";
                height: 50px;
            }
        }

        HorizontalBox {
            LineEdit {
                text: name;
                font-size: 40px;
                accepted(string) => {namechng(string);}
            }
        }

        GroupBox {
            vertical-stretch: 0;
                        ListView {
                has-focus: false;
                vertical-stretch: 1;
                for it in lines: 
                    HorizontalLayout {
                        LineEdit {
                            font-size: 35px;
                            read-only: true;
                            text: it.name;
                            width: 10%;
                        }
                        
                        LineEdit {
                            placeholder-text:"Enter";
                            text: it.value;
                            font-size: 35px;

                            width: 40%;
                            accepted(string) => {txtchng(it.name, string);}
                                }
                            }
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

fn serialize_cfg(cfg:&HashMap<u8,String>) {
    //let se = serde_json::to_writer(writer, cfg);
    //serde_json::from_reader(rdr)
    todo!();
}


fn update_current_cfg(cfg:&mut HashMap<u8,String>, new:Option<(i32,SharedString)>, app:&App) {

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

fn print_current_cfg(cfg:&HashMap<u8,String>) {
    println!("current cfg: {:?}", cfg);
}

fn send_cfg(cfg:&HashMap<u8,String>, socket:&BtStream) {

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


fn req_cfg(socket:&BtStream, loaded_config:&mut HashMap<u8,String>) {
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


    update_current_cfg(&mut loaded_config.lock().unwrap(), Option::None, &mut_app);

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