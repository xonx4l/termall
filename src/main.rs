use eframe::egui;

fn main() {
    unsafe{
          let res = nix::pty::forkpty(None, None);
    }
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My Terminal", native_options, Box::new(|cc| Ok(Box::new(Termali::new(cc)))));
}

#[derive(Default)]
struct Termali {}

impl Termali {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for Termali {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading("Hello World!");
       });
   }
}
