// tui.rs
//
#![allow(dead_code)]

use std::{
    fs::File,
    io::{self, stdout, Read, Write},
};
use termimad::crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode::*, KeyEvent},
    queue,
    style::Color::*,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use termimad::*;

fn view_area() -> Area {
    let mut area = Area::full_screen();
    area.pad_for_max_width(120);
    area
}

fn run_app(skin: MadSkin, markdown_content: String) -> Result<(), Error> {
    let mut w = stdout();
    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?;
    let mut view = MadView::from(markdown_content, view_area(), skin);
    loop {
        view.write_on(&mut w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                Char('k') => view.try_scroll_lines(-1),
                Char('j') => view.try_scroll_lines(1),
                PageUp => view.try_scroll_pages(-1),
                PageDown => view.try_scroll_pages(1),
                _ => break,
            },
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    terminal::disable_raw_mode()?;
    queue!(w, Show)?;
    queue!(w, LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}

fn read_markdown_file(file_path: &str) -> Result<String, io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn ui() -> Result<(), Error> {
    let skin = make_skin();

    match read_markdown_file("test.md") {
        Ok(md_contents) => run_app(skin, md_contents),
        Err(e) => {
            eprintln!("Failed to read markdown file: {}", e);
            Err(Error::from(e))
        }
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.align = Alignment::Center;
    skin
}
