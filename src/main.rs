use vrptw_code::VrpApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VRPTW Solver")
            .with_app_id("com.clement.vrptw")
            .with_inner_size([1200.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "VRPTW Solver",
        options,
        Box::new(|_cc| Ok(Box::new(VrpApp::new()))),
    )
}
