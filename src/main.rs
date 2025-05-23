use eframe::egui;
use::nix::unistd::ForkResult;
use std::{io::Read, fs::File, ffi::{CStr, CString}, os::fd::OwnedFd};

fn main() {
      let fd = unsafe{
          let res = nix::pty::forkpty(None, None).unwrap();
          match res.fork_result {
            ForkResult::Parent { .. } => (),
            ForkResult::Child => {
                let shell_name = CStr::from_bytes_with_nul(b"ash\0").expect("Should always have null terminator");
                nix::unistd::execvp::<CString>(shell_name, &[]).unwrap();
                return 
            }
          }
          res.master
    };

    
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My Terminal", native_options, Box::new( move |cc| Box::new(Termali::new(cc, fd))));
}

#[derive(Default)]
struct Termali {
    buf:Vec<u8>,
    fd: File,
}

impl Termali {
    fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Termali {
            buf: Vec::new(),
            fd:fd.into(),
        }
    }
}

impl eframe::App for Termali {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
       let mut buf = vec![0u8;4096];
       match self.fd.read(&mut buf){
        Ok(read_size) => {
        self.buf.extend_from_slice(&buf[0..read_size]);
        }
        Err(e) => {
            println!("failed to read: {e}");
        }

       }                  
       egui::CentralPanel::default().show(ctx, |ui| {
        unsafe{
           ui.label(std::str::from_utf8_unchecked(&self.buf))
        }
       });
   }
}
