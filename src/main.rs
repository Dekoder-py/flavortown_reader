use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{Event, KeyCode},
    },
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
};
use std::{env, str::FromStr};
use tui_markdown;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Resp {
    devlogs: Vec<Devlog>,
}

#[derive(Deserialize, Debug)]
struct Devlog {
    body: String,
    id: u32,
}

struct State {
    devlogs: Vec<Devlog>,
    selected: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenv::dotenv();
    let token = env::var("FT_API_KEY").expect("Failed to get API key from env");
    let url = "https://flavortown.hackclub.com/api/v1/devlogs";
    let client = reqwest::blocking::Client::new();
    let resp: Resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .expect("Failed to fetch")
        .json()
        .expect("Failed to parse");
    let mut state = State {
        devlogs: resp.devlogs,
        selected: 0,
    };
    ratatui::run(|terminal| app(terminal, &mut state))?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal, state: &mut State) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, state))?;
        match crossterm::event::read()? {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') {
                    break Ok(());
                }
                if key.code == KeyCode::Char('k') {
                    if state.selected != 0 {
                        state.selected -= 1;
                    }
                }

                if key.code == KeyCode::Char('j') {
                    if state.devlogs.get(state.selected + 1).is_some() {
                        state.selected += 1;
                    }
                }
            }
            _ => {}
        }
    }
}

fn render(frame: &mut Frame, state: &mut State) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(85),
        ])
        .split(frame.area());

    frame.render_widget(Paragraph::new("Flavortown Reader"), outer_layout[0]);

    let header = if let Some(devlog) = state.devlogs.get(state.selected) {
        format!("{}", devlog.id)
    } else {
        String::from_str("No devlogs").expect("idk why that failed bro")
    };

    frame.render_widget(Paragraph::new(header), outer_layout[1]);

    let text = if let Some(devlog) = state.devlogs.get(state.selected) {
        tui_markdown::from_str(&devlog.body)
    } else {
        tui_markdown::from_str("No devlogs")
    };

    frame.render_widget(Paragraph::new(text), outer_layout[2]);
}
