mod app;

fn main() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    let debug_pos = std::env::args().nth(1).unwrap_or_default();
    let in_debug = debug_pos == "dbg";

    if in_debug {
        native_options.viewport = egui::viewport::ViewportBuilder::default()
            .with_position((0.0, 0.0))
            .with_inner_size((1505.0, 1200.0));
    }

    let buf = match in_debug {
        true => Vec::from_iter((0..1000usize).map(|x| (x % 127) as u8).into_iter()),
        false => Vec::new(),
    };

    let _ = eframe::run_native(
        "Hexed.rs",
        native_options,
        Box::new(|_cc| Box::new(app::App::new(buf))),
    );

    Ok(())
}
