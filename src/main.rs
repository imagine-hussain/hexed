mod app;
mod file_source;
mod widgets;

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
    let native_options = eframe::NativeOptions::default();
    let args = Args::parse();

    let make_app: AppCreator = Box::new(move |_cc: &CreationContext| {
        let app = match args.filename {
            Some(filepath) => app::App::with_file(filepath.clone())
                .ok_or_else(|| anyhow!("Was unable to read file {filepath} set from command line"))
                .with_context(|| "Cannot create app")
                .unwrap(),
            None => app::App::new(),
        };
        Ok(Box::new(app))
    });

    let _ = eframe::run_native("Hexed.rs", native_options, make_app);

    Ok(())
}
