use crossbeam_channel::{Receiver, unbounded};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use ratatui::{buffer::Buffer, prelude::*, widgets::Block, widgets::Borders};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{
    io::{Read, Write},
    thread,
};
use vt100;

pub struct TermOverlay {
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    term: vt100::Parser,
    rows: u16,
    cols: u16,
    pub title: String,
    dirty: Arc<AtomicBool>,
}

impl TermOverlay {
    pub fn spawn(
        cmd: &str,
        args: &[&str],
        cols: u16,
        rows: u16,
        title: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut builder = CommandBuilder::new(cmd);
        for a in args {
            builder.arg(a);
        }
        builder.cwd(std::env::current_dir()?);
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");

        let child = pair.slave.spawn_command(builder)?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (tx, rx) = unbounded::<Vec<u8>>();
        let dirty = Arc::new(AtomicBool::new(true));
        let dirty_for_thread = dirty.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = tx.send(Vec::new());
                        break;
                    }
                    Ok(n) => {
                        let _ = tx.send(buf[..n].to_vec());
                        dirty_for_thread.store(true, Ordering::Release);
                    }
                    Err(_) => break,
                }
            }
        });

        // scrollback > 0 para histórico e "auto scroll down"
        let parser = vt100::Parser::new(rows, cols, 2000);

        Ok(Self {
            master: pair.master,
            _child: child,
            writer,
            rx,
            term: parser,
            rows,
            cols,
            title: title.into(),
            dirty,
        })
    }

    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::AcqRel)
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols == self.cols && rows == self.rows {
            return;
        }
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.term.set_size(rows, cols);
        self.cols = cols;
        self.rows = rows;
        self.dirty.store(true, Ordering::Release);
    }

    pub fn pump(&mut self) {
        while let Ok(chunk) = self.rx.try_recv() {
            if chunk.is_empty() {
                break;
            }
            self.term.process(&chunk);
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let cols = area.width.max(1);
        let rows = area.height.max(1);
        self.resize(cols, rows);
        self.pump();

        // desenha direto no buffer
        frame.render_widget(
            TermWidget {
                screen: self.term.screen().clone(),
                title: format!("{} [esc pra sair]", self.title),
            },
            area,
        );
    }

    pub fn send_str(&mut self, s: &str) {
        let _ = self.writer.write_all(s.as_bytes());
        let _ = self.writer.flush();
    }

    pub fn send_key(&mut self, key: crossterm::event::KeyCode) {
        match key {
            // enter como LF melhora compat com Textual e muitos TUIs
            crossterm::event::KeyCode::Enter => self.send_str("\r"),
            // backspace mais comum é DEL (0x7f), alguns apps ignoram 0x08
            crossterm::event::KeyCode::Backspace => self.send_str("\x7f"),
            crossterm::event::KeyCode::Tab => self.send_str("\t"),
            crossterm::event::KeyCode::Left => self.send_str("\x1b[D"),
            crossterm::event::KeyCode::Right => self.send_str("\x1b[C"),
            crossterm::event::KeyCode::Up => self.send_str("\x1b[A"),
            crossterm::event::KeyCode::Down => self.send_str("\x1b[B"),
            crossterm::event::KeyCode::Char(c) => {
                let mut s = String::new();
                s.push(c);
                self.send_str(&s);
            }
            _ => {}
        }
    }
}

struct TermWidget {
    screen: vt100::Screen,
    title: String,
}

impl Widget for TermWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // bordas
        let block = Block::default().title(self.title).borders(Borders::ALL);
        block.render(area, buf);
        let inner = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        // mostra sempre as ultimas linhas (comportamento de terminal)
        for y in 0..inner.height {
            for x in 0..inner.width {
                let by = inner.y + y;
                let bx = inner.x + x;

                let cell = buf.cell_mut((bx, by)).unwrap();
                cell.reset();
                cell.set_bg(Color::Black);

                // pega a célula da tela - o vt100 já cuida do scrollback
                if let Some(vc) = self.screen.cell(y, x) {
                    // símbolo
                    let s = vc.contents();
                    let ch = if s.is_empty() {
                        ' '
                    } else {
                        s.chars().next().unwrap_or(' ')
                    };
                    let mut tmp = [0u8; 4];
                    let sym = ch.encode_utf8(&mut tmp);
                    cell.set_symbol(sym);

                    // cores
                    let fg = vt100_to_ratatui_color(vc.fgcolor());
                    let bg = vt100_to_ratatui_color(vc.bgcolor());

                    if let Some(c) = fg {
                        cell.set_fg(c);
                    }
                    if let Some(c) = bg {
                        cell.set_bg(c);
                    }

                    // atributos
                    if vc.bold() {
                        cell.set_style(cell.style().add_modifier(Modifier::BOLD));
                    }
                    if vc.underline() {
                        cell.set_style(cell.style().add_modifier(Modifier::UNDERLINED));
                    }
                    if vc.inverse() {
                        cell.set_style(cell.style().add_modifier(Modifier::REVERSED));
                    }
                } else {
                    cell.set_symbol(" ");
                }
            }
        }
    }
}

// Helper: converte vt100::Color para ratatui::style::Color
fn vt100_to_ratatui_color(c: vt100::Color) -> Option<Color> {
    match c {
        vt100::Color::Default => None, // mantém a cor padrão do terminal
        vt100::Color::Idx(n) => {
            // cores ANSI indexadas (0-255)
            // 0-15: cores básicas (mapeia direto)
            // 16-255: paleta estendida
            match n {
                0 => Some(Color::Black),
                1 => Some(Color::Red),
                2 => Some(Color::Green),
                3 => Some(Color::Yellow),
                4 => Some(Color::Blue),
                5 => Some(Color::Magenta),
                6 => Some(Color::Cyan),
                7 => Some(Color::Gray), // ou White
                8 => Some(Color::DarkGray),
                9 => Some(Color::LightRed),
                10 => Some(Color::LightGreen),
                11 => Some(Color::LightYellow),
                12 => Some(Color::LightBlue),
                13 => Some(Color::LightMagenta),
                14 => Some(Color::LightCyan),
                15 => Some(Color::White),
                _ => Some(Color::Indexed(n)), // 16-255: usa a paleta 256 do ratatui
            }
        }
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
    }
}
