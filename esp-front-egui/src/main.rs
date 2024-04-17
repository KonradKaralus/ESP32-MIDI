#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
pub mod utils;

use std::{collections::HashMap, iter, sync::{Arc, Mutex}};
use indexmap::IndexMap;
use eframe::egui::{self, vec2, Button, Label, Pos2, Style, TextEdit};
use io_bluetooth::bt::{self, BtStream};

const ADDRESS:&str = "78:21:84:8c:71:2a";

const NUM_PEDALS:u8 = 6; 

fn main() -> Result<(), eframe::Error> {

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([1000.0, 690.0])
        .with_resizable(false)
        .with_position(Pos2::new(100f32, 100f32)),
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
            _cc.egui_ctx.set_pixels_per_point(2.5);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    columns: Arc<Mutex<IndexMap<u8, String>>>,
    socket: Option<BtStream>,
    custom_cmd:String,
    custom_pedal_nr:String,
    tempo:String,
    tempo_list:String,
    aliases:HashMap<String,String>
}

impl Default for MyApp {
    fn default() -> Self {
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
            socket: Option::from(socket),
            custom_cmd:"".to_string(),
            custom_pedal_nr:"".to_string(),
            tempo:"".to_string(),
            tempo_list:"".to_string(),
            aliases: MyApp::get_aliases()
        };

        res.req_cfg();        
        res
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                
                for (i,str) in self.columns.lock().unwrap().iter_mut() {
                    columns[0].add_sized(
                        vec2(20.0,20.0),
                        Label::new(i.to_string()));
                    columns[1].add_sized(
                        vec2(40.0,20.0),
                        TextEdit::singleline(str));
                }

            });

            ui.horizontal(|ui| {
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

            ui.separator();

            ui.horizontal(|ui| {
                if ui.add(Button::new("Tempos").min_size(vec2(80f32, 20f32))).clicked() {
                    self.send_tempo_list();
                }
                ui.add(TextEdit::singleline(&mut self.tempo_list))
            });

            ui.horizontal(|ui| {
                if ui.add(Button::new("Tempo").min_size(vec2(80f32, 20f32))).clicked() {
                    self.send_tempo_change();
                }
                ui.add(TextEdit::singleline(&mut self.tempo))
            });
            ui.horizontal(|ui| {

                if ui.add(Button::new("Hit Pedal").min_size(vec2(80f32, 20f32))).clicked() {
                    self.send_pedal_command();
                }

                ui.add(TextEdit::singleline(&mut self.custom_pedal_nr))
            });

            ui.horizontal(|ui| {
                if ui.add(Button::new("Command").min_size(vec2(80f32, 20f32))).clicked() {
                    self.send_midi_command();
                }
                ui.add(TextEdit::singleline(&mut self.custom_cmd))
            });
        });
    }
}