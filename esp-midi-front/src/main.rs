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

fn main() -> Result<(), std::io::Error> {
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

    app.run().unwrap();



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