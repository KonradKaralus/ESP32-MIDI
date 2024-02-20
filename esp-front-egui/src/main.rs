#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


pub mod utils;

const NUM_PEDALS:u8 = 4; 

const TEST:bool = false; 

const ADDRESS:&str = "78:21:84:8c:71:2a";

use std::{iter, sync::{Arc, Mutex}};

use indexmap::IndexMap;

use eframe::egui::{self, vec2, Label, Style, TextEdit};
use io_bluetooth::bt::{self, BtStream};

fn main() -> Result<(), eframe::Error> {

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(

        "My egui App",
        options,
        Box::new(|_cc| {
            let style = Style {
                visuals: egui::Visuals::dark(),
                ..Style::default()
            };
            _cc.egui_ctx.set_style(style);
            // This gives us image support:
            _cc.egui_ctx.set_pixels_per_point(3.2);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    columns: Arc<Mutex<IndexMap<u8, String>>>,
    socket: Option<BtStream>
}

impl Default for MyApp {
    fn default() -> Self {
        let v = vec!["CC1","PC2","CC3","PC5"];
        let mut map:IndexMap<u8, String> = IndexMap::with_capacity(NUM_PEDALS as usize); 
        for i in 1..=NUM_PEDALS {
            map.insert(i, v[(i-1) as usize].to_string());
        }
        Self {
            columns: Arc::new(Mutex::new(map)),
            socket: Option::None
        }
    }
}

impl MyApp {
    fn with_connection() -> Self {
        let loaded_config: IndexMap<u8,String> = IndexMap::with_capacity(NUM_PEDALS as usize); 

        let devices = bt::discover_devices().unwrap();
        let mut device_idx=0;
        for (idx,device) in devices.iter().enumerate() {
            if *device.to_string() == ADDRESS.to_string() {
                device_idx = idx;
                break;
            }
        }

        if devices.len() == 0 {
            panic!("no matching device");
        }

        let socket = BtStream::connect(iter::once(&devices[device_idx]), bt::BtProtocol::RFCOMM).unwrap();

        match socket.peer_addr() {
            Ok(_) => {},
            Err(err) => println!("An error occured while retrieving the peername: {:?}", err),
        }

        match socket.local_addr() {
            Ok(_) => {},
            Err(err) => println!("An error occured while retrieving the sockname: {:?}", err),
        }

        let mut res = Self {
            columns: Arc::new(Mutex::new(loaded_config)),
            socket: Option::from(socket)
        };
        res.req_cfg();
        
        res
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("My egui Application");
            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            ui.columns(2, |columns| {
                

                for (i,str) in self.columns.lock().unwrap().iter_mut() {
                    columns[0].add_sized(
                        vec2(40.0,20.0),
                        Label::new(i.to_string()));
                    columns[1].add_sized(
                        vec2(40.0,20.0),
                        TextEdit::singleline(str));
                }

            });

            ui.horizontal(|ui| {
                if ui.button("Print").clicked() {
                    self.print_current_cfg();
                }
                if ui.button("Send").clicked() {
                    self.send_cfg();
                }
                if ui.button("Save").clicked() {
                    self.serialize_cfg();
                }
                if ui.button("Load").clicked() {
                    self.load_cfg();
                }
                if ui.button("Get").clicked() {
                    self.req_cfg();
                }
            });
        });
    }
}