#![allow(unused_variables)]
#![allow(dead_code)]

use egui::{CentralPanel, ScrollArea};
use egui_file_dialog::FileDialog;

use crate::file_source::FileWatcher;
use crate::widgets::FrameCounter;

pub struct App {
    menu_text_input: String,
    active_file: Option<String>,
    file_dialog: egui_file_dialog::FileDialog,
    file_watcher: FileWatcher,
    framecounter: FrameCounter,
}

impl App {
    pub fn new() -> Self {
        Self {
            menu_text_input: String::with_capacity(128),
            active_file: None,
            file_dialog: FileDialog::new(),
            file_watcher: FileWatcher::new(),
            framecounter: FrameCounter::new(),
        }
    }

    pub fn with_file(filepath: String) -> Option<Self> {
        let mut app = Self {
            menu_text_input: String::with_capacity(128),
            active_file: None,
            file_dialog: FileDialog::new(),
            file_watcher: FileWatcher::new(),
            framecounter: FrameCounter::new(),
        };

        match app.try_update_active_file(filepath) {
            true => Some(app),
            false => None,
        }
    }
}

struct HexView<'a> {
    file: &'a mut FileWatcher,
}

const NIBBLE: usize = 16;

impl<'a> HexView<'a> {
    pub fn new(file: &'a mut FileWatcher) -> Self {
        Self { file }
    }

    const ROW_HEIGHT: f32 = 15.0;

    pub fn show(&mut self, ui: &mut egui::Ui) {
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
        let total_rows = self.file.file_len() / NIBBLE;

        let mut row_buf = [0; NIBBLE];

        let _ = {};

        let rendered_rows =
            ScrollArea::vertical().show_rows(ui, Self::ROW_HEIGHT, total_rows, |ui, row_range| {
                for row in row_range.clone() {
                    ui.horizontal(|ui| {
                        self.nth_row(row, &mut row_buf);

                        address_col(row, ui);
                        row_buf.iter().for_each(|&n| hex_box(n, ui));
                        hex_to_ascii_separator(ui);
                        row_buf.iter().for_each(|&n| ascii_box(n, ui));
                        ui.monospace(" ");
                    });
                }
                row_range
            });
    }

    pub fn nth_row(&mut self, row: usize, buf: &mut [u8; NIBBLE]) {
        let start = row * NIBBLE;
        self.file.get_range_within_page(start..start + NIBBLE, buf);
    }
}

impl App {
    fn try_update_active_file(&mut self, raw_filepath: String) -> bool {
        match self.file_watcher.try_update_active_file(raw_filepath) {
            Some(newly_watched_path) => {
                self.active_file = Some(newly_watched_path);
                true
            }
            None => false,
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            let file_res = ui.button("File Picker");
            ui.separator();
            if file_res.clicked() {
                self.file_dialog.select_file();
            }

            // Update the dialog and check if the user selected a file
            self.file_dialog.update(ctx);
            if let Some(path) = self.file_dialog.take_selected() {
                // self.selected_file = Some(path.to_path_buf());
                let s = path.to_str().unwrap_or_default().to_string();
                self.try_update_active_file(s);
            }

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
                self.try_update_active_file(self.menu_text_input.clone());
            }
            ui.separator();
            ui.label("Framerate: ");
            ui.label(self.framecounter.fps().to_string());
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.framecounter.register_tick();
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            self.menu_bar(ctx, ui);
        });

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                HexView::new(&mut self.file_watcher).show(ui);
            });
        });

        ctx.request_repaint();
        // ctx.request_repaint_after(Duration::from_millis(100));
    }
}
