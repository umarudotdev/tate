use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

const DIM: Color = Color::Rgb(76, 86, 106);

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        let syntax_set = two_face::syntax::extra_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-eighties.dark"].clone();
        Highlighter { syntax_set, theme }
    }

    pub fn highlight(&self, lines: &[String], extension: &str) -> Vec<Line<'static>> {
        let width = line_number_width(lines.len());

        let syntax = match self.syntax_set.find_syntax_by_extension(extension) {
            Some(s) => s,
            None => return plain_lines(lines, width),
        };

        let mut h = syntect::easy::HighlightLines::new(syntax, &self.theme);
        let mut result = Vec::with_capacity(lines.len());

        for (i, line) in lines.iter().enumerate() {
            let gutter = format!(" {:>width$}  ", i + 1);
            let mut spans = vec![Span::styled(gutter, Style::default().fg(DIM))];

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

fn plain_lines(lines: &[String], width: usize) -> Vec<Line<'static>> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            Line::from(vec![
                Span::styled(format!(" {:>width$}  ", i + 1), Style::default().fg(DIM)),
                Span::raw(l.to_string()),
            ])
        })
        .collect()
}

fn line_number_width(total: usize) -> usize {
    if total == 0 {
        1
    } else {
        ((total as f64).log10().floor() as usize) + 1
    }
}
