use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{Event, KeyCode},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{env, str::FromStr};

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

    frame.render_widget(
        Paragraph::new("Flavortown Reader | Something's Cooking!")
            .block(Block::new().bold().fg(Color::Cyan).borders(Borders::ALL)),
        outer_layout[0],
    );

    let header = if let Some(devlog) = state.devlogs.get(state.selected) {
        format!("Devlog #{}", devlog.id)
    } else {
        String::from_str("No devlogs").expect("idk why that failed bro")
    };

    frame.render_widget(
        Paragraph::new(header).block(Block::new().fg(Color::Blue).borders(Borders::ALL)),
        outer_layout[1],
    );

    let text = if let Some(devlog) = state.devlogs.get(state.selected) {
        render_markdown(&devlog.body)
    } else {
        render_markdown("No Devlogs!")
    };

    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(Block::new().borders(Borders::ALL)),
        outer_layout[2],
    );
}

fn render_markdown(input: &str) -> Text<'static> {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span, Text};

    let parser = Parser::new_ext(input, Options::all());

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_style = match level {
                    HeadingLevel::H1 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    HeadingLevel::H2 => Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default().add_modifier(Modifier::BOLD),
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                lines.push(Line::from(current_spans.clone()));
                lines.push(Line::default()); // blank line after heading
                current_spans.clear();
                current_style = Style::default();
            }
            Event::Start(Tag::Strong) => {
                current_style = current_style
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Magenta);
            }
            Event::End(TagEnd::Strong) => {
                current_style = current_style.remove_modifier(Modifier::BOLD);
            }
            Event::Start(Tag::Emphasis) => {
                current_style = current_style.add_modifier(Modifier::ITALIC);
            }
            Event::End(TagEnd::Emphasis) => {
                current_style = current_style.remove_modifier(Modifier::ITALIC);
            }
            Event::End(TagEnd::Paragraph) => {
                lines.push(Line::from(current_spans.clone()));
                lines.push(Line::default());
                current_spans.clear();
            }
            Event::Text(t) => {
                current_spans.push(Span::styled(t.to_string(), current_style));
            }
            Event::SoftBreak => {
                lines.push(Line::from(current_spans.clone()));
                current_spans.clear();
            }
            Event::HardBreak => {
                lines.push(Line::from(current_spans.clone()));
                current_spans.clear();
            }

            Event::Start(Tag::List(_)) => {
                // nothing needed on start
            }
            Event::End(TagEnd::List(_)) => {
                lines.push(Line::default()); // blank line after list
            }
            Event::Start(Tag::Item) => {
                current_spans.push(Span::raw("• "));
            }
            Event::End(TagEnd::Item) => {
                lines.push(Line::from(current_spans.clone()));
                current_spans.clear();
            }
            _ => {}
        }
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    Text::from(lines)
}
