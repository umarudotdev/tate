use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

const DEFAULT_GUTTER: Color = Color::Rgb(76, 86, 106);

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
    theme_set: ThemeSet,
    theme_names: Vec<String>,
    current_theme_index: usize,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new("base16-eighties.dark")
    }
}

pub fn load_theme_set() -> ThemeSet {
    let mut theme_set: ThemeSet = two_face::theme::extra().into();
    for (name, bytes) in [
        (
            "Rosé Pine",
            &include_bytes!("../themes/rose-pine.tmTheme")[..],
        ),
        (
            "Rosé Pine Moon",
            &include_bytes!("../themes/rose-pine-moon.tmTheme")[..],
        ),
        (
            "Rosé Pine Dawn",
            &include_bytes!("../themes/rose-pine-dawn.tmTheme")[..],
        ),
        (
            "Tokyo Night",
            &include_bytes!("../themes/tokyonight_night.tmTheme")[..],
        ),
        (
            "Tokyo Night Storm",
            &include_bytes!("../themes/tokyonight_storm.tmTheme")[..],
        ),
    ] {
        if let Ok(theme) = ThemeSet::load_from_reader(&mut std::io::Cursor::new(bytes)) {
            theme_set.themes.insert(name.to_string(), theme);
        }
    }
    theme_set
}

impl Highlighter {
    pub fn new(theme_name: &str) -> Self {
        let syntax_set = two_face::syntax::extra_newlines();
        let theme_set = load_theme_set();

        let mut theme_names: Vec<String> = theme_set.themes.keys().cloned().collect();
        theme_names.sort();
        let current_theme_index = theme_names
            .iter()
            .position(|n| n == theme_name)
            .unwrap_or(0);
        let theme = theme_set
            .themes
            .get(theme_name)
            .or_else(|| theme_set.themes.get("base16-eighties.dark"))
            .cloned()
            .unwrap_or_else(|| theme_set.themes.values().next().unwrap().clone());
        Highlighter {
            syntax_set,
            theme,
            theme_set,
            theme_names,
            current_theme_index,
        }
    }

    pub fn current_theme_index(&self) -> usize {
        self.current_theme_index
    }

    pub fn theme_names(&self) -> &[String] {
        &self.theme_names
    }

    pub fn theme_count(&self) -> usize {
        self.theme_names.len()
    }

    pub fn set_theme_by_index(&mut self, index: usize) {
        self.current_theme_index = index % self.theme_names.len();
        let name = &self.theme_names[self.current_theme_index];
        self.theme = self.theme_set.themes[name].clone();
    }

    pub fn highlight(&self, lines: &[String], extension: &str) -> Vec<Line<'static>> {
        let width = line_number_width(lines.len());
        let gutter_color = gutter_color_from_theme(&self.theme);

        let syntax = match self.syntax_set.find_syntax_by_extension(extension) {
            Some(s) => s,
            None => return plain_lines(lines, width, gutter_color),
        };

        let mut h = syntect::easy::HighlightLines::new(syntax, &self.theme);
        let mut result = Vec::with_capacity(lines.len());

        for (i, line) in lines.iter().enumerate() {
            let gutter = format!(" {:>width$}  ", i + 1);
            let mut spans = vec![Span::styled(gutter, Style::default().fg(gutter_color))];

            match h.highlight_line(line, &self.syntax_set) {
                Ok(regions) => {
                    for (style, text) in regions {
                        let fg =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        spans.push(Span::styled(text.to_string(), Style::default().fg(fg)));
                    }
                }
                Err(_) => {
                    spans.push(Span::raw(line.to_string()));
                }
            }

            result.push(Line::from(spans));
        }

        result
    }
}

fn plain_lines(lines: &[String], width: usize, gutter_color: Color) -> Vec<Line<'static>> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            Line::from(vec![
                Span::styled(
                    format!(" {:>width$}  ", i + 1),
                    Style::default().fg(gutter_color),
                ),
                Span::raw(l.to_string()),
            ])
        })
        .collect()
}

fn gutter_color_from_theme(theme: &syntect::highlighting::Theme) -> Color {
    theme
        .settings
        .gutter_foreground
        .map(|c| Color::Rgb(c.r, c.g, c.b))
        .unwrap_or(DEFAULT_GUTTER)
}

fn line_number_width(total: usize) -> usize {
    if total == 0 {
        1
    } else {
        total.ilog10() as usize + 1
    }
}
