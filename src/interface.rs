use ansi_to_tui::IntoText;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    input,
    rss::{example_feed, Article, Website},
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    // Fix, this can be better
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // same here
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

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub(crate) struct App {
    websites: StatefulList<Website>,
    articles: StatefulList<Article>,
    scroll: u16,
}

impl App {
    pub fn new(config: Config) -> App {
        let mut websites = vec![
            example_feed("https://everythingchanges.us/feed.xml").unwrap(),
            example_feed("https://charity.wtf/feed/").unwrap(),
        ];

        //TODO load websites from config insead
        config
            .subscriptions
            .iter()
            .for_each(|site| websites.push(example_feed(site).unwrap()));

        App {
            websites: StatefulList::with_items(websites),
            articles: StatefulList::new(),
            scroll: 0,
        }
    }

    fn load_articles(&mut self) {
        if let Some(index) = self.websites.state.selected() {
            self.articles = StatefulList::with_items(self.websites.items[index].articles.clone());
        }
    }

    fn clear_articles(&mut self) {
        self.articles = StatefulList::new();
    }

    fn scroll_down(&mut self, width: u16, length: u16) {
        if let Some(index) = self.articles.state.selected() {
            // move this elsewhere you are doing it twice (here and in read UI)
            let raw_html = self.articles.items[index].content.clone();
            let parsed_to_markdown = html2md::parse_html(&raw_html);
            let markdown_to_terminal = termimad::inline(&parsed_to_markdown).to_string();

            let content_length_md = markdown_to_terminal.chars().count();
            let line_count_md = markdown_to_terminal.lines().count();

            let content_length =
                (content_length_md / Into::<usize>::into(width)) + line_count_md - 50;
            if Into::<usize>::into(self.scroll) <= content_length {
                self.scroll += 1;
            } else {
                self.scroll += 0;
            }
        } else {
            self.scroll += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        } else {
            self.reset_scroll();
        }
    }

    fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    fn on_tick(&mut self) {}
}

pub(crate) fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('a') => {
                        let sites = input::main().unwrap();
                        for site in sites {
                            crate::config::update_or_store(site.clone()).unwrap();
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.websites.unselect();
                        return Ok(());
                    }
                    KeyCode::Left => {
                        app.clear_articles();
                        app.websites.unselect()
                    }
                    KeyCode::Down => {
                        app.websites.next();
                        app.load_articles()
                    }
                    KeyCode::Up => {
                        app.websites.previous();
                        app.load_articles()
                    }
                    KeyCode::Char('h') => 'help_loop: loop {
                        terminal.draw(|f| help_ui(f))?;

                        if crossterm::event::poll(timeout)? {
                            if let Event::Key(key) = event::read()? {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        break 'help_loop;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    },
                    KeyCode::Right | KeyCode::Enter => 'article_select: loop {
                        terminal.draw(|f| ui(f, &mut app))?;
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Left => app.articles.unselect(),
                                KeyCode::Down => app.articles.next(),
                                KeyCode::Up => app.articles.previous(),
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.articles.unselect();
                                    break 'article_select;
                                }
                                KeyCode::Char('h') => 'help_loop: loop {
                                    terminal.draw(|f| help_ui(f))?;

                                    if crossterm::event::poll(timeout)? {
                                        if let Event::Key(key) = event::read()? {
                                            match key.code {
                                                KeyCode::Char('q') | KeyCode::Esc => {
                                                    break 'help_loop;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                },
                                KeyCode::Right | KeyCode::Enter => {
                                    if let Some(article) = app.articles.state.selected() {
                                        'read: loop {
                                            terminal.draw(|f| {
                                                read_ui(f, &app, &app.articles.items[article])
                                            })?;
                                            // get the width here so we can compute how far the user
                                            // can scroll to based off their terminal size since we wrap the text
                                            let width = terminal.get_frame().size().width;
                                            let length = terminal.get_frame().size().bottom();

                                            let timeout = tick_rate
                                                .checked_sub(last_tick.elapsed())
                                                .unwrap_or_else(|| Duration::from_secs(0));
                                            if crossterm::event::poll(timeout)? {
                                                if let Event::Key(key) = event::read()? {
                                                    match key.code {
                                                        KeyCode::Up => app.scroll_up(),
                                                        KeyCode::Down => {
                                                            app.scroll_down(width, length)
                                                        }
                                                        KeyCode::Esc | KeyCode::Char('q') => {
                                                            app.reset_scroll();
                                                            break 'read;
                                                        }
                                                        KeyCode::Char('h') => 'help_loop: loop {
                                                            terminal.draw(|f| help_ui(f))?;

                                                            if crossterm::event::poll(timeout)? {
                                                                if let Event::Key(key) =
                                                                    event::read()?
                                                                {
                                                                    match key.code {
                                                                        KeyCode::Char('q')
                                                                        | KeyCode::Esc => {
                                                                            break 'help_loop;
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                    }
                                }
                                _ => {}
                            }
                        }
                    },
                    // all other keys do nothing
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn read_ui<B: Backend>(f: &mut Frame<B>, app: &App, article: &Article) {
    let size = f.size();

    let block = Block::default();
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
        .split(size);

    let raw_html = article.content.clone();
    let parsed_to_markdown = html2md::parse_html(&raw_html);
    let markdown_to_terminal = termimad::inline(&parsed_to_markdown).to_string();
    let ansi_to_tui = markdown_to_terminal.into_text().unwrap();

    let create_block = |title| {
        Block::default().borders(Borders::ALL).title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
    };
    let paragraph = Paragraph::new(ansi_to_tui)
        .block(create_block(article.title.clone()))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));

    f.render_widget(paragraph, chunks[0]);
}

fn help_ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();

    let block = Block::default();
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
        .split(size);

    let create_block = |title| {
        Block::default().borders(Borders::ALL).title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
    };

    let paragraph = Paragraph::new("hello".into_text().unwrap())
        .block(create_block("Key Shortcuts <Press Esc to close>"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, chunks[0]);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with divided horizontal screen space (20/80)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let sites: Vec<ListItem> = app
        .websites
        .items
        .iter()
        .map(|site| {
            // adds the website name
            ListItem::new(site.name.clone())
                .style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(sites)
        .block(Block::default().borders(Borders::ALL).title("Website"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.websites.state);

    let entries: Vec<ListItem> = {
        app.articles
            .items
            .iter()
            .map(|article| {
                ListItem::new(format!(
                    "{}\n\t\t{}",
                    article.title.clone(),
                    article.updated_at.clone()
                ))
                .style(Style::default().fg(Color::Black).bg(Color::White))
            })
            .collect()
    };

    let entries_list = List::new(entries)
        .block(Block::default().borders(Borders::ALL).title("Articles"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(entries_list, chunks[1], &mut app.articles.state);
}
