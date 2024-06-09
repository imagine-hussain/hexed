#![allow(unused_variables)]
#![allow(dead_code)]

use egui::{CentralPanel, ScrollArea, TextBuffer};

pub struct App {
    buf: Vec<u8>,
    menu_text_input: String,
    active_file: Option<String>,
}

impl App {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            menu_text_input: String::with_capacity(128),
            active_file: None,
        }
    }
}

struct HexView<'a> {
    buf: &'a [u8],
}

const NIBBLE: usize = 16;

impl<'a> HexView<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let hex_box = |val: u8, ui: &mut egui::Ui| {
            ui.monospace(format!("{val:02x} "));
        };

        let address_col = |row: usize, ui: &mut egui::Ui| {
            let address = row * NIBBLE;
            ui.monospace(format!("{address:#08x} | "));
        };
        let hex_to_ascii_separator = |ui: &mut egui::Ui| {
            ui.monospace(" | ");
        };
        let ascii_box = |n: u8, ui: &mut egui::Ui| {
            let repr = match n {
                (32..=126) => n as char,
                _ => '.',
            };
            ui.monospace(String::from(repr));
        };

        ui.horizontal(|ui| {
            ui.monospace("Address  | ");
            for col in 0..NIBBLE as u8 {
                hex_box(col, ui);
            }
            hex_to_ascii_separator(ui);
            ui.monospace("Ascii");
        });
        ui.separator();
        let total_rows = self.buf.len() / NIBBLE;
        ScrollArea::vertical().show_rows(ui, 15.0, total_rows, |ui, row_range| {
            for row in row_range {
                ui.horizontal(|ui| {
                    address_col(row, ui);
                    self.nth_row(row).iter().for_each(|&n| hex_box(n, ui));
                    hex_to_ascii_separator(ui);
                    self.nth_row(row).iter().for_each(|&n| ascii_box(n, ui));
                    ui.monospace(" ");
                });
            }
        });
    }

    pub fn nth_row(&self, row: usize) -> &[u8] {
        let start = row * NIBBLE;
        &self.buf[start..start + NIBBLE]
    }
}
impl App {
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            let file_res = ui.button("File");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.menu_text_input).hint_text(
                    self.active_file
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("No Active File"),
                ),
            );
            let set_active_res = ui.button("Update Current File");
            if set_active_res.clicked() {
                // Change the active_file
                let expanded_path_raw = match shellexpand::full(self.menu_text_input.as_str()) {
                    Ok(path) => path.to_string(),
                    Err(_) => String::new(),
                };

                let path = std::path::Path::new(expanded_path_raw.as_str());

                if path.exists() && path.is_file() {
                    self.active_file = Some(expanded_path_raw.to_string());
                    dbg!("exists", path);
                } else {
                    dbg!("no exist", path);
                }
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            self.menu_bar(ui);
        });

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                HexView::new(&self.buf).show(ui);
            });
        });
    }
}
