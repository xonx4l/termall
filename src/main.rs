use eframe::egui;
use std::{
    io::Read,
    fs::File,
    ffi::{CStr, CString},
    os::fd::OwnedFd, 
};

fn get_char_size(ctx: &egui::Context) -> (f32 , f32) {
    let font_id = ctx.style().text_styles[&egui::TextStyle::Monospace].clone();
    let spacing = &ctx.style().spacing;
    println!("item_spacing: {:?}", spacing.item_spacing);
    let (width , height) = ctx.fonts(move |fonts| {

        let layout = fonts.layout(
            "@".to_string(),
            font_id,
            egui::Color32::default(),
            f32::INFINITY);

        (layout.mesh_bounds.width(), layout.mesh_bounds.height())    
    });

    (width , height)
}

fn character_to_cursor_offset(character_pos: &(usize, usize), character_size: &(f32, f32), content: &[u8]) -> (f32 , f32) {
    let content_by_lines: Vec<&[u8]> = content.split( |b| *b == b'\n').collect();
    let num_lines = content_by_lines.len();
    let x_offset = character_pos.0 as f32 * character_size.0;
    let y_offset = (character_pos.1 as i64 - num_lines as i64) as f32 * character_size.1;
    (x_offset, y_offset)
}

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
        " My Termall",
        native_options,
        Box::new(move |cc| Ok(Box::new(Termali::new(cc, fd)))),
    );
}

// ---

struct Termali {
    buf: Vec<u8>,
    cursor_pos: (usize, usize),
    character_size: Option<(f32, f32)>,
    fd: File,
}

impl Termali {
    fn new(_cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        _cc.egui_ctx.style_mut(|style| {
            style.override_text_style = Some(egui::TextStyle::Monospace);
        });
        let flags = nix::fcntl::fcntl(&fd, nix::fcntl::FcntlArg::F_GETFL).unwrap();
        let mut flags = nix::fcntl::OFlag::from_bits(flags & nix::fcntl::OFlag::O_ACCMODE.bits()).unwrap();
        flags.set(nix::fcntl::OFlag::O_NONBLOCK, true);

        
        nix::fcntl::fcntl(&fd, nix::fcntl::FcntlArg::F_SETFL(flags)).unwrap();

        Termali {
            buf: Vec::new(),
            fd: fd.into(), 
            cursor_pos: (0, 0),
            character_size: None,
        }
    }
}

// ---

impl eframe::App for Termali {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.character_size.is_none() {
            self.character_size = Some(get_char_size(ctx));
              println!("character size: {:?}", self.character_size);
        }
        let mut buf = vec![0u8; 4096];
        match self.fd.read(&mut buf) {
            Ok(read_size) => {
                let incoming = &buf[0..read_size];
                for c in incoming {
                    match c {
                        b'\n' => self.cursor_pos = (0, self.cursor_pos.1 + 1),
                        _ => self.cursor_pos = (self.cursor_pos.0 + 1, self.cursor_pos.1),
                    }
                }
                self.buf.extend_from_slice(incoming);
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    eprintln!("Failed to read from PTY: {e}");
                }
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.input(|input_state| {
                for event in &input_state.events {
                    let text = match event {
                        egui::Event::Text(text) => text,
                        egui::Event::Key { key : egui::Key::Enter, pressed: true,  ..} => {
                            "\n"
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

            
        let response = unsafe {
                ui.label(std::str::from_utf8_unchecked(&self.buf))
            
        };

        let bottom = response.rect.bottom();
        let left = response.rect.left() + 3.0;

        let painter = ui.painter();
        let character_size =  self.character_size.as_ref().unwrap();
        let cursor_offset = character_to_cursor_offset(&self.cursor_pos, character_size, &self.buf);
        painter.rect_filled( egui::Rect::from_min_size(
            egui::pos2(left + cursor_offset.0, bottom + cursor_offset.1),
            egui::vec2(character_size.0, character_size.1),
            ),
            0.0,
            egui::Color32::GRAY);
        });
    
    }
}