use eframe::egui;
use std::{
    io::Read,
    fs::File,
    ffi::{CStr, CString},
    os::fd::{OwnedFd, AsRawFd}, 
};

fn main() {
    let fd = unsafe {
        let res = nix::pty::forkpty(None, None).unwrap();

        match res {
            nix::pty::ForkptyResult::Parent { child: _, master } => { 
                master
            }
            nix::pty::ForkptyResult::Child => {
                let shell_name = CStr::from_bytes_with_nul(b"/bin/bash\0")
                    .expect("Should always have null terminator");

                nix::unistd::execvp::<CString>(
                    shell_name,
                    &[CString::from_vec_unchecked(shell_name.to_bytes().to_vec())],
                )
                .unwrap();

                panic!("execvp failed");
            }
        }
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My Terminal",
        native_options,
        Box::new(move |cc| Ok(Box::new(Termali::new(cc, fd)))),
    );
}

// ---

struct Termali {
    buf: Vec<u8>,
    fd: File,
}

impl Termali {
    fn new(_cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        Termali {
            buf: Vec::new(),
            fd: fd.into(), // Convert OwnedFd into a File
        }
    }
}

// ---

impl eframe::App for Termali {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       let mut buf = vec![0u8; 4096];
       match self.fd.read(&mut buf){
        Ok(read_size) => {
             self.buf.extend_from_slice(&buf[0..read_size]);
        }
        Err(e) => {
            // Check for EIO (Input/output error) which can happen when the PTY closes.
            // Other errors like EAGAIN (Resource temporarily unavailable) might also occur
            // if the read is non-blocking and no data is ready.
            if e.kind() != std::io::ErrorKind::WouldBlock { // Only print if not a "would block" error
                eprintln!("Failed to read from PTY: {e}");
            }
          }

       }
       egui::CentralPanel::default().show(ctx, |ui| {
        ui.input(|input_state| {
            for event in &input_state.events {
                let text = match event {
                    egui::Event::Text(text) => text,
                    egui::Event::Key { key : egui::Key::Enter,  ..} => {
                        "/n"
                    }
                    _ => "",
                };

                let bytes = text.as_bytes();
                let mut to_write: &[u8] = bytes;

                while to_write.len() > 0 {
                    
                    let written = nix::unistd::write(&self.fd, to_write).unwrap(); 
                    
                    to_write = &to_write[written..];
                }
            }
        });
        unsafe {
           ui.label(std::str::from_utf8_unchecked(&self.buf))
          }
       });
   }
}