use eframe::egui;
use::nix::unistd::ForkResult;
use std::ffi::{Cstr, Cstring};

fn main() {
      let fd = unsafe{
          let res = nix::pty::forkpty(None, None);
          match res.fork.result{
            ForkResult::Parent {..} => (),
            ForkResult::Child => {
                let shell_name = Cstr::from_bytes_with_null(b"ash\0").expect("Should always have null terminator");
                nix::unistd::execvp::<Cstring>(shell_name, &[]).unwrap();
                return 
            }
          }
          res.master
    }
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My Terminal", native_options, Box::new( move |cc| Ok(Box::new(Termali::new(cc, fd)))));
}

#[derive(Default)]
struct Termali {
    buf:String,
    fd: OwnedFd,
}

impl Termali {
    fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Termali {
            buf: String::new(),
            fd,
        }
    }
}

impl eframe::App for Termali {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading("Hello World!");
       });
   }
}
