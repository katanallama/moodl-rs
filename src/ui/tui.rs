// tui.rs
//
use {
    minimad::{TextTemplate, OwningTemplateExpander},
    termimad::crossterm::style::Color::*,
    termimad::*,
};
use std::io::{stdout, Write};
use termimad::crossterm::{
    cursor::{ Hide, Show},
    event::{
        self,
        Event,
        KeyEvent,
        KeyCode::*,
    },
    queue,
    terminal::{
        self,
        Clear,
        ClearType,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

static MODTEMPLATE: &str = r#"
-----------
# ${app-name}

${module-rows
-----------
## ${module-name}
${module-description}
}
-----------
"#;

/// a struct to illustrate several ways to format its information
pub struct ParsedModule {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}

pub struct _ParsedGrade {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}

fn view_area() -> Area {
    let mut area = Area::full_screen();
    area.pad_for_max_width(120); // we don't want a too wide text column
    area
}

pub fn ui(modules: &[ParsedModule]) -> Result<(), Error> {
    // fill an expander with data
    let mut expander = OwningTemplateExpander::new();
    expander
        .set("app-name", "ENSE400")
        .set("app-version", "0.01")
        .set_md("dynamic", "filled_by_**template**");
    for module in modules {
        expander.sub("module-rows")
            .set("module-name", &module.name)
            .set_md("module-description", &module.description);
    }
    let mut w = stdout(); // we could also have used stderr
    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?; // hiding the cursor


    // let template = TextTemplate::from(MODTEMPLATE);

    let (width, _) = terminal_size();
    let template_md: String = MODTEMPLATE.into();
    let template = TextTemplate::from(&*template_md);
    let text = expander.expand(&template);
    let skin = make_skin();

    let fmt_text = FmtText::from_text(&skin, text, Some(width as usize));
    // print!("{}", fmt_text);


    // let mut view = MadView::from(MD.to_owned(), view_area(), skin);
    let mut view = MadView::from(fmt_text.to_string().to_owned(), view_area(), skin);
    loop {
        view.write_on(&mut w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent{code, ..})) => {
                match code {
                    Char('k') => view.try_scroll_lines(-1),
                    Char('j') => view.try_scroll_lines(1),
                    PageUp => view.try_scroll_pages(-1),
                    PageDown => view.try_scroll_pages(1),
                    _ => break,
                }
            }
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    terminal::disable_raw_mode()?;
    queue!(w, Show)?; // we must restore the cursor
    queue!(w, LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(AnsiValue(178));
    skin.headers[2].set_fg(gray(22));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    // skin.code_block.align = Alignment::Center;
    skin
}
