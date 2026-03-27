use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Entry {
    File(PathBuf),
    Symbol { path: PathBuf, name: String },
    Range { path: PathBuf, start: u32, end: u32 },
}

impl Entry {
    pub fn parse(s: &str) -> Result<Self, ParseEntryError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseEntryError::Empty);
        }

        if let Some((path, name)) = s.split_once("::") {
            let path = path.trim();
            let name = name.trim();
            if path.is_empty() {
                return Err(ParseEntryError::EmptyPath);
            }
            if name.is_empty() {
                return Err(ParseEntryError::EmptySymbol);
            }
            return Ok(Entry::Symbol {
                path: PathBuf::from(path),
                name: name.to_string(),
            });
        }

        if let Some((path, range)) = s.rsplit_once(':') {
            if let Some((start_str, end_str)) = range.split_once('-') {
                if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                    if path.is_empty() {
                        return Err(ParseEntryError::EmptyPath);
                    }
                    if start == 0 || end == 0 {
                        return Err(ParseEntryError::InvalidRange);
                    }
                    if start > end {
                        return Err(ParseEntryError::InvalidRange);
                    }
                    return Ok(Entry::Range {
                        path: PathBuf::from(path),
                        start,
                        end,
                    });
                }
            }
        }

        Ok(Entry::File(PathBuf::from(s)))
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            Entry::File(p) => p,
            Entry::Symbol { path, .. } => path,
            Entry::Range { path, .. } => path,
        }
    }

    pub fn to_deck_line(&self) -> String {
        match self {
            Entry::File(p) => p.display().to_string(),
            Entry::Symbol { path, name } => format!("{}::{}", path.display(), name),
            Entry::Range { path, start, end } => {
                format!("{}:{}-{}", path.display(), start, end)
            }
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_deck_line())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEntryError {
    Empty,
    EmptyPath,
    EmptySymbol,
    InvalidRange,
}

impl fmt::Display for ParseEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "entry is empty"),
            Self::EmptyPath => write!(f, "path is empty"),
            Self::EmptySymbol => write!(f, "symbol name is empty"),
            Self::InvalidRange => write!(
                f,
                "invalid line range (use path:start-end, both > 0, start <= end)"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_entry() {
        let e = Entry::parse("src/auth/login.ts").unwrap();
        assert_eq!(e, Entry::File(PathBuf::from("src/auth/login.ts")));
    }

    #[test]
    fn parse_symbol_entry() {
        let e = Entry::parse("src/auth/login.ts::authenticate").unwrap();
        assert_eq!(
            e,
            Entry::Symbol {
                path: PathBuf::from("src/auth/login.ts"),
                name: "authenticate".to_string(),
            }
        );
    }

    #[test]
    fn parse_range_entry() {
        let e = Entry::parse("src/styles.css:5-16").unwrap();
        assert_eq!(
            e,
            Entry::Range {
                path: PathBuf::from("src/styles.css"),
                start: 5,
                end: 16,
            }
        );
    }

    #[test]
    fn parse_empty_is_error() {
        assert_eq!(Entry::parse(""), Err(ParseEntryError::Empty));
        assert_eq!(Entry::parse("  "), Err(ParseEntryError::Empty));
    }

    #[test]
    fn parse_empty_path_is_error() {
        assert_eq!(Entry::parse("::foo"), Err(ParseEntryError::EmptyPath));
    }

    #[test]
    fn parse_empty_symbol_is_error() {
        assert_eq!(
            Entry::parse("src/file.rs::"),
            Err(ParseEntryError::EmptySymbol)
        );
    }

    #[test]
    fn parse_range_zero_start_is_error() {
        assert_eq!(
            Entry::parse("file.css:0-5"),
            Err(ParseEntryError::InvalidRange)
        );
    }

    #[test]
    fn parse_range_start_greater_than_end_is_error() {
        assert_eq!(
            Entry::parse("file.css:10-5"),
            Err(ParseEntryError::InvalidRange)
        );
    }

    #[test]
    fn parse_file_with_colon_in_path_not_a_range() {
        let e = Entry::parse("C:\\Users\\file.rs").unwrap();
        assert_eq!(e, Entry::File(PathBuf::from("C:\\Users\\file.rs")));
    }

    #[test]
    fn round_trip_symbol() {
        let line = "src/auth/login.ts::authenticate";
        let entry = Entry::parse(line).unwrap();
        assert_eq!(entry.to_deck_line(), line);
    }

    #[test]
    fn round_trip_range() {
        let line = "src/styles.css:5-16";
        let entry = Entry::parse(line).unwrap();
        assert_eq!(entry.to_deck_line(), line);
    }
}
