// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod command;
mod connection;
mod icon;
mod tests;
mod utils;

use connection::{check_connection_status, get_connection, Connection};
use eframe::egui::{self, vec2, IconData, Label, Pos2, RadioButton, Style, TextEdit};
use icon::ARR;
use indexmap::IndexMap;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub const ADDRESS: &str = "78:21:84:8c:71:2a";

const NUM_PEDALS: usize = 6;
const CC_SEP: char = '/';

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 690.0])
            .with_resizable(false)
            .with_position(Pos2::new(100f32, 100f32))
            .with_icon(IconData {
                rgba: ARR.to_vec(),
                width: 256,
                height: 256,
            }),
        ..Default::default()
    };

    eframe::run_native(
        "MIDI-Controller",
        options,
        Box::new(|_cc| {
            let style = Style {
                visuals: egui::Visuals::dark(),
                ..Style::default()
            };
            _cc.egui_ctx.set_style(style);
            _cc.egui_ctx.set_pixels_per_point(2.5);
            Box::<ControllerApp>::default()
        }),
    )
}

struct ControllerApp {
    columns: Arc<Mutex<IndexMap<u8, String>>>,
    _tempo: String,
    aliases: HashMap<String, String>,
    aliases_rev: HashMap<String, String>,
    connection: Connection,
}

impl Default for ControllerApp {
    fn default() -> Self {
        let loaded_config: IndexMap<u8, String> = IndexMap::with_capacity(NUM_PEDALS);

        let c = get_connection();
        let c2 = c.clone();

        thread::spawn(|| {
            check_connection_status(c2);
        });

        let aliases = ControllerApp::get_aliases();

        Self {
            columns: Arc::new(Mutex::new(loaded_config)),
            _tempo: "".to_string(),
            aliases: aliases.0,
            aliases_rev: aliases.1,
            connection: c,
        }
    }
}

impl eframe::App for ControllerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(250));
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_newly_connected() {
                self.req_cfg();
            }

            ui.horizontal(|ui| {
                ui.add(RadioButton::new(self.is_connected(), ""));
                if !self.is_connected() {
                    if ui.button("Connect").clicked() {
                        self.connection.lock().try_connect = true;
                    }
                }
                ui.label(self.get_connection_status());
            });

            if !self.is_connected() {
                return;
            }

            ui.columns(2, |columns| {
                for (i, str) in self.columns.lock().unwrap().iter_mut() {
                    columns[0].add_sized(vec2(20.0, 20.0), Label::new(i.to_string()));
                    columns[1].add_sized(vec2(40.0, 20.0), TextEdit::singleline(str));
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

            ui.label(format!("Format: CC|PC<val>{}<on>,<off>", CC_SEP));
        });
    }
}
