use crossbeam_channel::{Receiver, unbounded};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use ratatui::{
    buffer::Buffer,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    io::{Read, Write},
    thread,
};
use vt100;

pub struct TermOverlay {
    // PTY
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,

    // emulador de terminal
    term: vt100::Parser,

    // geometria atual
    rows: u16,
    cols: u16,

    // titulo opcional
    pub title: String,
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

        // garanta cwd e TERM
        let cwd = std::env::current_dir()?;
        builder.cwd(cwd);
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");

        let child = pair.slave.spawn_command(builder)?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        // thread que le do PTY e envia pro canal
        let (tx, rx) = unbounded::<Vec<u8>>();
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = tx.send(Vec::new()); // EOF
                        break;
                    }
                    Ok(n) => {
                        let _ = tx.send(buf[..n].to_vec());
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        // emulador com o mesmo tamanho do PTY
        let parser = vt100::Parser::new(rows, cols, 0);

        Ok(Self {
            master: pair.master,
            _child: child,
            writer,
            rx,
            term: parser,
            rows,
            cols,
            title: title.into(),
        })
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
    }

    // drena dados pendentes do PTY e alimenta o emulador
    pub fn pump(&mut self) {
        while let Ok(chunk) = self.rx.try_recv() {
            if chunk.is_empty() {
                // EOF
                break;
            }
            self.term.process(&chunk);
        }
    }

    // desenha o conteudo do emulador na área dada
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // mantém o PTY do mesmo tamanho do retangulo disponível
        let cols = area.width.max(1);
        let rows = area.height.max(1);
        self.resize(cols, rows);

        // atualiza o emulador com dados novos
        self.pump();

        // converte as celulas do emulador em Lines/Spans do ratatui
        let screen = self.term.screen();
        let mut lines: Vec<Line> = Vec::with_capacity(rows as usize);
        for r in 0..rows {
            let mut spans = Vec::with_capacity(cols as usize);
            for c in 0..cols {
                if let Some(cell) = screen.cell(r, c) {
                    let ch = cell.contents();
                    let mut style = Style::default();
                    let fg = cell.fgcolor();
                    if let vt100::Color::Rgb(r, g, b) = fg {
                        style = style.fg(Color::Rgb(r, g, b));
                    }
                    let bg = cell.bgcolor();
                    if let vt100::Color::Rgb(r, g, b) = bg {
                        style = style.bg(Color::Rgb(r, g, b));
                    }
                    if cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if cell.inverse() {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    spans.push(Span::styled(ch.to_string(), style));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
            lines.push(Line::from(spans));
        }

        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);
        let p = Paragraph::new(lines).block(block);
        frame.render_widget(Clear, area); // limpa atras do overlay
        frame.render_widget(p, area);
    }

    // envia teclas pro processo (opcional)
    pub fn send_str(&mut self, s: &str) {
        let _ = self.writer.write_all(s.as_bytes());
        let _ = self.writer.flush();
    }

    pub fn send_key(&mut self, key: crossterm::event::KeyCode) {
        match key {
            crossterm::event::KeyCode::Enter => self.send_str("\r"),
            crossterm::event::KeyCode::Backspace => self.send_str("\x08"),
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

// widget Clear para limpar a área do overlay
struct Clear;
impl Widget for Clear {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).unwrap();
                cell.reset();
                cell.set_symbol(" ");
            }
        }
    }
}
