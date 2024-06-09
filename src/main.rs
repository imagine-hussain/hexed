mod app;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    let buf = Vec::from_iter((0..1000usize).map(|x| (x % 127) as u8).into_iter());

    let _ = eframe::run_native(
        "Hexed.rs",
        native_options,
        Box::new(|_cc| Box::new(app::App::new(buf))),
    );

    Ok(())
}
