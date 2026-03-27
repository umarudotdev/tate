use std::io::{self, Stdout};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

use tate_core::grade::Grade;
use tate_core::review::SkipReason;

#[derive(Clone, Copy)]
struct Theme {
    border: Color,
    text: Color,
    hint: Color,
    question: Color,
    answer: Color,
    grade_again: Color,
    grade_hard: Color,
    grade_good: Color,
    grade_easy: Color,
    accent: Color,
    progress: Color,
    success: Color,
    warning: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            border: Color::DarkGray,
            text: Color::White,
            hint: Color::DarkGray,
            question: Color::Cyan,
            answer: Color::Green,
            grade_again: Color::Red,
            grade_hard: Color::Yellow,
            grade_good: Color::Green,
            grade_easy: Color::Magenta,
            accent: Color::Cyan,
            progress: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
        }
    }
}

pub enum UserInput {
    Grade(Grade),
    Quit,
}

pub struct ReviewTui {
    terminal: Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    highlighter: crate::highlight::Highlighter,
    theme: Theme,
    entry: String,
    file_ext: String,
    review_num: u32,
    lapses: u32,
    source_lines: Vec<String>,
    source_scroll: u16,
    question: String,
    answer: Option<String>,
    answer_revealed: bool,
    graded: bool,
    selected_grade: usize,
    next_review: Option<String>,
    current_card: u32,
    total_cards: u32,
    theme_picker_open: bool,
    theme_picker_state: ListState,
    theme_picker_saved_index: usize,
}

impl ReviewTui {
    pub fn new(syntect_theme: &str) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(ReviewTui {
            terminal,
            highlighter: crate::highlight::Highlighter::new(syntect_theme),
            theme: Theme::default(),
            entry: String::new(),
            file_ext: String::new(),
            review_num: 0,
            lapses: 0,
            source_lines: Vec::new(),
            source_scroll: 0,
            question: String::new(),
            answer: None,
            answer_revealed: false,
            graded: false,
            selected_grade: 2,
            next_review: None,
            current_card: 0,
            total_cards: 0,
            theme_picker_open: false,
            theme_picker_state: ListState::default(),
            theme_picker_saved_index: 0,
        })
    }

    pub fn set_progress(&mut self, total: u32) {
        self.total_cards = total;
        self.current_card = 0;
    }

    pub fn show_card(
        &mut self,
        entry: &str,
        review_num: u32,
        lapses: u32,
        source: Option<&str>,
        question: &str,
    ) {
        self.current_card += 1;
        self.entry = entry.to_string();
        self.lapses = lapses;
        let path_part = entry.split("::").next().unwrap_or(entry);
        self.file_ext = std::path::Path::new(path_part)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        self.review_num = review_num;
        self.source_lines = source
            .map(|s| s.lines().map(|l| l.to_string()).collect())
            .unwrap_or_default();
        self.source_scroll = 0;
        self.question = question.to_string();
        self.answer = None;
        self.answer_revealed = false;
        self.graded = false;
        self.selected_grade = 2;
        self.next_review = None;
        self.render().ok();
    }

    pub fn reveal_answer(&mut self, answer: &str) -> bool {
        self.answer = Some(answer.to_string());
        self.answer_revealed = false;
        self.render().ok();

        loop {
            if let Ok(Event::Key(key)) = event::read() {
                if self.handle_scroll(key.code, key.modifiers) {
                    self.render().ok();
                    continue;
                }
                if key.code == KeyCode::Char(' ') {
                    break;
                }
                if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.run_theme_picker();
                    continue;
                }
                if key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Char('Q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    return true;
                }
            }
        }

        self.answer_revealed = true;
        self.render().ok();
        false
    }

    pub fn prompt_grade(&mut self) -> UserInput {
        self.render().ok();

        let grades = [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy];

        loop {
            if let Ok(Event::Key(key)) = event::read() {
                if self.handle_scroll(key.code, key.modifiers) {
                    self.render().ok();
                    continue;
                }
                match key.code {
                    KeyCode::Char('1') => return UserInput::Grade(Grade::Again),
                    KeyCode::Char('2') => return UserInput::Grade(Grade::Hard),
                    KeyCode::Char('3') => return UserInput::Grade(Grade::Good),
                    KeyCode::Char('4') => return UserInput::Grade(Grade::Easy),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        return UserInput::Grade(grades[self.selected_grade])
                    }
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                        self.selected_grade = if self.selected_grade == 0 {
                            3
                        } else {
                            self.selected_grade - 1
                        };
                        self.render().ok();
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                        self.selected_grade = (self.selected_grade + 1) % 4;
                        self.render().ok();
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.run_theme_picker();
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => return UserInput::Quit,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return UserInput::Quit
                    }
                    _ => {}
                }
            }
        }
    }

    fn run_theme_picker(&mut self) {
        self.theme_picker_saved_index = self.highlighter.current_theme_index();
        self.theme_picker_open = true;
        self.theme_picker_state
            .select(Some(self.theme_picker_saved_index));
        self.render().ok();

        loop {
            if let Ok(Event::Key(key)) = event::read() {
                let count = self.highlighter.theme_count();
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        let cur = self.theme_picker_state.selected().unwrap_or(0);
                        let next = (cur + 1) % count;
                        self.theme_picker_state.select(Some(next));
                        self.highlighter.set_theme_by_index(next);
                        self.render().ok();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let cur = self.theme_picker_state.selected().unwrap_or(0);
                        let next = if cur == 0 { count - 1 } else { cur - 1 };
                        self.theme_picker_state.select(Some(next));
                        self.highlighter.set_theme_by_index(next);
                        self.render().ok();
                    }
                    KeyCode::Enter => {
                        self.theme_picker_open = false;
                        self.render().ok();
                        return;
                    }
                    KeyCode::Esc => {
                        self.highlighter
                            .set_theme_by_index(self.theme_picker_saved_index);
                        self.theme_picker_open = false;
                        self.render().ok();
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn show_skip(&mut self, entry: &str, reason: &SkipReason) {
        let reason_str = match reason {
            SkipReason::FileNotFound => "file not found",
            SkipReason::SymbolNotFound { .. } => "symbol not found",
            SkipReason::ParseFailed => "parse failed",
        };
        self.current_card += 1;
        let t = self.theme;
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  {} ", entry),
                        Style::default().fg(t.warning),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  Reason: {}", reason_str),
                        Style::default().fg(t.hint),
                    )),
                ];
                let paragraph = Paragraph::new(text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Skipped ")
                        .title_style(Style::default().fg(t.warning)),
                );
                frame.render_widget(paragraph, area);
            })
            .ok();

        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    pub fn show_summary(&mut self, reviewed: u32, skipped: u32) {
        let t = self.theme;
        self.terminal
            .draw(|frame| {
                let area = frame.area();

                let chunks = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Fill(1),
                ])
                .split(area);

                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        " Session Complete ",
                        Style::default().fg(t.success).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(
                            "Reviewed {} card{}",
                            reviewed,
                            if reviewed == 1 { "" } else { "s" }
                        ),
                        Style::default().fg(t.text),
                    )),
                ];
                if skipped > 0 {
                    lines.push(Line::from(Span::styled(
                        format!("Skipped {}", skipped),
                        Style::default().fg(t.hint),
                    )));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Press any key to exit",
                    Style::default().fg(t.hint),
                )));

                let paragraph = Paragraph::new(lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Summary ")
                            .title_style(Style::default().fg(t.success)),
                    )
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, chunks[1]);
            })
            .ok();

        loop {
            if let Ok(Event::Key(_)) = event::read() {
                break;
            }
        }
    }

    pub fn show_next_review(&mut self, date: &str) {
        self.graded = true;
        self.next_review = Some(date.to_string());
        self.render().ok();
    }

    fn render(&mut self) -> io::Result<()> {
        let entry = self.entry.clone();
        let review_num = self.review_num;
        let lapses = self.lapses;
        let highlighted_lines = self
            .highlighter
            .highlight(&self.source_lines, &self.file_ext);
        let source_scroll = self.source_scroll;
        let total_source_lines = self.source_lines.len();
        let question = self.question.clone();
        let answer = self.answer.clone();
        let answer_revealed = self.answer_revealed;
        let graded = self.graded;
        let next_review = self.next_review.clone();
        let current_card = self.current_card;
        let total_cards = self.total_cards;
        let t = self.theme;
        let selected_grade = self.selected_grade;
        let theme_picker_open = self.theme_picker_open;
        let theme_names: Vec<String> = if theme_picker_open {
            self.highlighter.theme_names().to_vec()
        } else {
            Vec::new()
        };
        let mut picker_state = self.theme_picker_state.clone();

        self.terminal.draw(|frame| {
            let area = frame.area();

            let q_height: u16 = if !answer_revealed && answer.is_some() {
                5
            } else {
                4
            };
            let a_height: u16 = if answer_revealed { 4 } else { 0 };
            let bottom_height: u16 = if graded || answer_revealed || answer.is_none() {
                3
            } else {
                0
            };

            let chunks = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(q_height),
                Constraint::Length(a_height),
                Constraint::Length(bottom_height),
            ])
            .split(area);

            if total_cards > 0 {
                let ratio = current_card as f64 / total_cards as f64;
                let label = format!("{}/{}", current_card, total_cards);
                let gauge = Gauge::default()
                    .ratio(ratio.min(1.0))
                    .label(label)
                    .gauge_style(Style::default().fg(t.progress));
                frame.render_widget(gauge, chunks[0]);
            }

            let scroll_info = if total_source_lines > 0 {
                let visible_end = (source_scroll as usize
                    + chunks[1].height.saturating_sub(2) as usize)
                    .min(total_source_lines);
                format!(
                    " {}-{}/{} ",
                    source_scroll + 1,
                    visible_end,
                    total_source_lines
                )
            } else {
                String::new()
            };

            let title = format!(" {} ", entry);
            let code = Paragraph::new(highlighted_lines.clone()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(Style::default().fg(t.accent))
                    .title_bottom(
                        Line::from(Span::styled(scroll_info, Style::default().fg(t.hint)))
                            .alignment(Alignment::Right),
                    )
                    .border_style(Style::default().fg(t.border)),
            );
            let code = code.scroll((source_scroll, 0));
            frame.render_widget(code, chunks[1]);

            let q_title = if review_num == 0 && lapses > 0 {
                " Lapsed ".to_string()
            } else if review_num == 0 {
                " First review ".to_string()
            } else {
                format!(" Review #{} ", review_num)
            };

            let mut q_lines: Vec<Line> = vec![Line::from(Span::styled(
                format!("  {}", question),
                Style::default().fg(t.text),
            ))];

            if !answer_revealed && answer.is_some() {
                q_lines.push(Line::from(""));
                q_lines.push(Line::from(Span::styled(
                    "  [space] flip",
                    Style::default().fg(t.hint).add_modifier(Modifier::ITALIC),
                )));
            }

            let q_pane = Paragraph::new(q_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(q_title)
                        .title_style(Style::default().fg(t.question))
                        .border_style(Style::default().fg(t.border)),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(q_pane, chunks[2]);

            if answer_revealed {
                let mut a_lines: Vec<Line> = Vec::new();
                if let Some(ref ans) = answer {
                    a_lines.push(Line::from(Span::styled(
                        format!("  {}", ans),
                        Style::default().fg(t.text),
                    )));
                }

                let a_pane = Paragraph::new(a_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Answer ")
                            .title_style(Style::default().fg(t.answer))
                            .border_style(Style::default().fg(t.border)),
                    )
                    .wrap(Wrap { trim: false });
                frame.render_widget(a_pane, chunks[3]);
            }

            if bottom_height > 0 && graded {
                let date_text = next_review
                    .as_deref()
                    .map(|d| format!("Next review: {}", d))
                    .unwrap_or_default();
                let bar = Paragraph::new(Line::from(Span::styled(
                    date_text,
                    Style::default().fg(t.success).add_modifier(Modifier::BOLD),
                )))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(t.border)),
                )
                .alignment(Alignment::Center);
                frame.render_widget(bar, chunks[4]);
            } else if bottom_height > 0 {
                let options: [(Color, &str); 4] = [
                    (t.grade_again, " [1] Again "),
                    (t.grade_hard, " [2] Hard "),
                    (t.grade_good, " [3] Good "),
                    (t.grade_easy, " [4] Easy "),
                ];
                let mut grade_spans = Vec::new();
                for (i, (color, label)) in options.iter().enumerate() {
                    let mut style = Style::default().fg(*color).add_modifier(Modifier::BOLD);
                    if i == selected_grade {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    grade_spans.push(Span::raw("  "));
                    grade_spans.push(Span::styled(*label, style));
                }
                grade_spans.push(Span::raw("    "));
                grade_spans.push(Span::styled("[q] Quit", Style::default().fg(t.hint)));
                let grade_bar = Paragraph::new(Line::from(grade_spans))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(t.border)),
                    )
                    .alignment(Alignment::Center);
                frame.render_widget(grade_bar, chunks[4]);
            }

            if theme_picker_open {
                let popup_area = centered_rect(60, 70, area);
                frame.render_widget(Clear, popup_area);

                let items: Vec<ListItem> = theme_names
                    .iter()
                    .map(|name| {
                        ListItem::new(Span::styled(
                            format!(" {}", name),
                            Style::default().fg(t.text),
                        ))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Theme Picker ")
                            .title_style(Style::default().fg(t.accent))
                            .title_bottom(
                                Line::from(Span::styled(
                                    " [j/k] navigate  [enter] select  [esc] cancel ",
                                    Style::default().fg(t.hint),
                                ))
                                .alignment(Alignment::Center),
                            )
                            .border_style(Style::default().fg(t.border)),
                    )
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(t.accent)
                            .add_modifier(Modifier::BOLD),
                    );

                frame.render_stateful_widget(list, popup_area, &mut picker_state);
            }
        })?;

        if theme_picker_open {
            self.theme_picker_state = picker_state;
        }

        Ok(())
    }

    fn scroll_up(&mut self) {
        self.source_scroll = self.source_scroll.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        let max = self.source_lines.len().saturating_sub(3) as u16;
        if self.source_scroll < max {
            self.source_scroll += 1;
        }
    }

    fn scroll_half_up(&mut self) {
        self.source_scroll = self.source_scroll.saturating_sub(10);
    }

    fn scroll_half_down(&mut self) {
        let max = self.source_lines.len().saturating_sub(3) as u16;
        self.source_scroll = (self.source_scroll + 10).min(max);
    }

    fn scroll_top(&mut self) {
        self.source_scroll = 0;
    }

    fn scroll_bottom(&mut self) {
        self.source_scroll = self.source_lines.len().saturating_sub(3) as u16;
    }

    fn handle_scroll(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        match (code, modifiers) {
            (KeyCode::Up, _)
            | (KeyCode::Char('k'), KeyModifiers::NONE)
            | (KeyCode::Char('w'), KeyModifiers::NONE) => self.scroll_up(),
            (KeyCode::Down, _)
            | (KeyCode::Char('j'), KeyModifiers::NONE)
            | (KeyCode::Char('s'), KeyModifiers::NONE) => self.scroll_down(),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.scroll_half_up(),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.scroll_half_down(),
            (KeyCode::Char('g'), KeyModifiers::NONE) => self.scroll_top(),
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => self.scroll_bottom(),
            _ => return false,
        }
        true
    }
}

impl Drop for ReviewTui {
    fn drop(&mut self) {
        terminal::disable_raw_mode().ok();
        io::stdout().execute(LeaveAlternateScreen).ok();
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(v[1])[1]
}
