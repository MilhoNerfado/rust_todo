use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, ModifierKeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

#[derive(Copy, Clone)]
struct UiConfig {
    projects: bool,
    typebox: bool,
}

trait BoolToggleExt {
    fn toggle(&mut self);
}

impl BoolToggleExt for bool {
    fn toggle(&mut self) {
        *self = !*self;
    }
}

struct TodoItem {
    is_done: bool,
    title: String,
}

impl ToString for TodoItem {
    fn to_string(&self) -> String {
        String::from(format!(
            "[{}] {}",
            if self.is_done { " " } else { "x" },
            self.title
        ))
    }
}

struct TodoList {
    state: ListState,
    items: Vec<TodoItem>,
}

impl TodoList {
    fn new(items: Vec<TodoItem>) -> TodoList {
        TodoList {
            state: ListState::default(),
            items,
        }
    }

    fn default() -> TodoList {
        TodoList {
            state: ListState::default(),
            items: vec![
                TodoItem {
                    is_done: false,
                    title: "Hello world".to_string(),
                },
                TodoItem {
                    is_done: true,
                    title: "Hello again".to_string(),
                },
                TodoItem {
                    is_done: true,
                    title: "Bye!!".to_string(),
                },
                TodoItem {
                    is_done: false,
                    title: "A line here...".to_string(),
                },
                TodoItem {
                    is_done: true,
                    title: "What should i do?? \nthis?".to_string(),
                },
                TodoItem {
                    is_done: true,
                    title: "Uno\nDos\nTres!".to_string(),
                },
            ],
        }
    }

    fn remove(&mut self, pos: usize) -> &mut TodoList {
        self.items.remove(pos);
        self
    }

    fn add(&mut self, item: TodoItem) -> &mut TodoList {
        self.items.push(item);
        self
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselec(&mut self) {
        self.state.select(None);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut ui_config = UiConfig {
        projects: true,
        typebox: true,
    };

    let mut todo_list = TodoList::default();

    loop {
        terminal.draw(|f| ui(f, ui_config, &mut todo_list))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::End => ui_config.projects.toggle(),
                KeyCode::Home => ui_config.typebox.toggle(),
                KeyCode::Down => todo_list.next(),
                KeyCode::Up => todo_list.previous(),
                KeyCode::Left => todo_list.unselec(),
                _ => (),
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, config: UiConfig, data: &mut TodoList) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    let size = f.size();

    // Surrounding block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Main block with round corners ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Thick);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints(
            [
                Constraint::Percentage(if config.typebox { 90 } else { 100 }),
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(f.size());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(if config.projects { 80 } else { 100 }),
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let items: Vec<ListItem> = data
        .items
        .iter()
        .map(|i| {
            ListItem::new(i.to_string().clone())
                .style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, top_chunks[0], &mut data.state);

    // // Top left inner block with green background
    // let block = Block::default()
    //     .style(Style::default().bg(Color::Reset))
    //     .borders(Borders::ALL)
    //     .border_type(BorderType::Thick)
    //     .border_style(Style::default().fg(Color::Cyan));
    // f.render_widget(block, top_chunks[0]);

    // Top right inner block with styled title aligned to the right
    let block = Block::default()
        .title(Span::styled(
            "Projects",
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);
    f.render_widget(block, top_chunks[1]);

    let block = Block::default().title("Text").borders(Borders::ALL);
    f.render_widget(block, chunks[1])
}
