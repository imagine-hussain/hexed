mod app;
mod file_source;

use anyhow::{anyhow, Context};
use clap::Parser;
use eframe::{AppCreator, CreationContext};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    filename: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    let debug_pos = std::env::args().nth(1).unwrap_or_default();
    let in_debug = debug_pos == "dbg";

    if in_debug {
        native_options.viewport = egui::viewport::ViewportBuilder::default()
            .with_position((0.0, 0.0))
            .with_inner_size((1505.0, 1200.0));
    }

    let args = Args::parse();

    let make_app: AppCreator = Box::new(move |_cc: &CreationContext| {
        let app = match args.filename {
            Some(filepath) => app::App::with_file(filepath.clone())
                .ok_or_else(|| anyhow!("Was unable to read file {filepath} set from command line"))
                .with_context(|| "Cannot create app")
                .unwrap(),
            None => app::App::new(),
        };
        Box::new(app)
    });

    let _ = eframe::run_native("Hexed.rs", native_options, make_app);

    Ok(())
}
