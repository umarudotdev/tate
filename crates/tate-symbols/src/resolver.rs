use std::path::Path;

use tree_sitter::Parser;

use crate::error::SymbolError;
use crate::languages::language_for_extension;

pub fn resolve_symbol(path: &Path, symbol_name: &str) -> Result<Vec<u8>, SymbolError> {
    let source = read_file(path)?;
    let config = get_language_config(path)?;

    let mut parser = Parser::new();
    parser
        .set_language(&config.language)
        .map_err(|_| SymbolError::ParseFailed {
            path: path.to_path_buf(),
        })?;

    let tree = parser
        .parse(&source, None)
        .ok_or_else(|| SymbolError::ParseFailed {
            path: path.to_path_buf(),
        })?;

    let mut found_symbols = Vec::new();
    let mut cursor = tree.walk();

    find_symbol_in_tree(
        &mut cursor,
        &source,
        config.symbol_node_types,
        symbol_name,
        &mut found_symbols,
    )
    .ok_or_else(|| SymbolError::SymbolNotFound {
        path: path.to_path_buf(),
        name: symbol_name.to_string(),
        found: found_symbols,
    })
}

pub fn list_symbols(path: &Path) -> Result<Vec<String>, SymbolError> {
    let source = read_file(path)?;
    let config = get_language_config(path)?;

    let mut parser = Parser::new();
    parser
        .set_language(&config.language)
        .map_err(|_| SymbolError::ParseFailed {
            path: path.to_path_buf(),
        })?;

    let tree = parser
        .parse(&source, None)
        .ok_or_else(|| SymbolError::ParseFailed {
            path: path.to_path_buf(),
        })?;

    let mut symbols = Vec::new();
    collect_symbols(&tree, &source, config.symbol_node_types, &mut symbols);
    Ok(symbols)
}

pub fn hash_file(path: &Path) -> Result<String, SymbolError> {
    let source = read_file(path)?;
    Ok(blake3::hash(&source).to_hex().to_string())
}

pub fn hash_symbol(path: &Path, symbol_name: &str) -> Result<String, SymbolError> {
    let body = resolve_symbol(path, symbol_name)?;
    Ok(blake3::hash(&body).to_hex().to_string())
}

pub fn resolve_range(path: &Path, start: u32, end: u32) -> Result<Vec<u8>, SymbolError> {
    let source = read_file(path)?;
    let lines: Vec<&[u8]> = source.split(|&b| b == b'\n').collect();
    if (end as usize) > lines.len() {
        return Err(SymbolError::Io {
            path: path.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "line range {}-{} exceeds file length {}",
                    start,
                    end,
                    lines.len()
                ),
            ),
        });
    }
    let selected: Vec<u8> = lines[(start as usize - 1)..(end as usize)]
        .iter()
        .enumerate()
        .flat_map(|(i, line)| {
            let mut v = line.to_vec();
            if i < (end - start) as usize {
                v.push(b'\n');
            }
            v
        })
        .collect();
    Ok(selected)
}

pub fn hash_range(path: &Path, start: u32, end: u32) -> Result<String, SymbolError> {
    let body = resolve_range(path, start, end)?;
    Ok(blake3::hash(&body).to_hex().to_string())
}

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(crate::languages::is_supported)
        .unwrap_or(false)
}

pub fn supports_symbols(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(crate::languages::supports_symbols)
        .unwrap_or(false)
}
fn read_file(path: &Path) -> Result<Vec<u8>, SymbolError> {
    std::fs::read(path).map_err(|e| SymbolError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn get_language_config(path: &Path) -> Result<crate::languages::LanguageConfig, SymbolError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    language_for_extension(ext).ok_or_else(|| SymbolError::UnsupportedLanguage {
        ext: ext.to_string(),
    })
}

fn find_symbol_in_tree(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    symbol_types: &[&str],
    target: &str,
    found_symbols: &mut Vec<String>,
) -> Option<Vec<u8>> {
    loop {
        let node = cursor.node();

        if symbol_types.contains(&node.kind()) {
            if let Some(name) = extract_symbol_name(&node, source) {
                found_symbols.push(name.clone());
                if name == target {
                    let start = node.start_byte();
                    let end = node.end_byte();
                    return Some(source[start..end].to_vec());
                }
            }
        }

        if cursor.goto_first_child() {
            if let Some(result) =
                find_symbol_in_tree(cursor, source, symbol_types, target, found_symbols)
            {
                return Some(result);
            }
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            return None;
        }
    }
}

fn collect_symbols(
    tree: &tree_sitter::Tree,
    source: &[u8],
    symbol_types: &[&str],
    symbols: &mut Vec<String>,
) {
    let mut cursor = tree.walk();
    collect_symbols_recursive(&mut cursor, source, symbol_types, symbols);
}

fn collect_symbols_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    symbol_types: &[&str],
    symbols: &mut Vec<String>,
) {
    loop {
        let node = cursor.node();

        if symbol_types.contains(&node.kind()) {
            if let Some(name) = extract_symbol_name(&node, source) {
                symbols.push(name);
            }
        }

        if cursor.goto_first_child() {
            collect_symbols_recursive(cursor, source, symbol_types, symbols);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            return;
        }
    }
}

fn extract_symbol_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        let text = &source[name_node.start_byte()..name_node.end_byte()];
        return std::str::from_utf8(text).ok().map(|s| s.to_string());
    }

    if let Some(decl) = node.child_by_field_name("declarator") {
        if let Some(name) = extract_identifier_from(&decl, source) {
            return Some(name);
        }
    }

    extract_identifier_from(node, source)
}

fn extract_identifier_from(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let kind = node.kind();
    if matches!(
        kind,
        "identifier"
            | "property_identifier"
            | "type_identifier"
            | "value_name"
            | "module_name"
            | "type_name"
    ) {
        let text = &source[node.start_byte()..node.end_byte()];
        return std::str::from_utf8(text).ok().map(|s| s.to_string());
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(name) = extract_identifier_from(&child, source) {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_file(ext: &str, content: &str) -> tempfile::TempPath {
        let mut f = NamedTempFile::with_suffix(&format!(".{}", ext)).unwrap();
        write!(f, "{}", content).unwrap();
        f.into_temp_path()
    }

    #[test]
    fn resolve_rust_function() {
        let path = write_temp_file(
            "rs",
            r#"
fn hello() {
    println!("hello");
}

fn world() {
    println!("world");
}
"#,
        );
        let body = resolve_symbol(&path, "hello").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("println!(\"hello\")"));
        assert!(!text.contains("println!(\"world\")"));
    }

    #[test]
    fn list_rust_symbols() {
        let path = write_temp_file(
            "rs",
            r#"
fn alpha() {}
struct Beta {}
fn gamma() {}
"#,
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(symbols.contains(&"alpha".to_string()));
        assert!(symbols.contains(&"Beta".to_string()));
        assert!(symbols.contains(&"gamma".to_string()));
    }

    #[test]
    fn symbol_not_found_with_suggestions() {
        let path = write_temp_file("rs", "fn existing() {}\n");
        let err = resolve_symbol(&path, "nonexistent").unwrap_err();
        if let SymbolError::SymbolNotFound { found, .. } = err {
            assert!(found.contains(&"existing".to_string()));
        } else {
            panic!("expected SymbolNotFound, got {:?}", err);
        }
    }

    #[test]
    fn hash_file_is_stable() {
        let path = write_temp_file("rs", "fn main() {}\n");
        let h1 = hash_file(&path).unwrap();
        let h2 = hash_file(&path).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn hash_symbol_is_stable() {
        let path = write_temp_file("rs", "fn main() {}\n");
        let h1 = hash_symbol(&path, "main").unwrap();
        let h2 = hash_symbol(&path, "main").unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn unsupported_extension() {
        let path = write_temp_file("xyz", "content");
        let err = resolve_symbol(&path, "foo").unwrap_err();
        assert!(matches!(err, SymbolError::UnsupportedLanguage { .. }));
    }

    #[test]
    fn resolve_rust_struct() {
        let path = write_temp_file(
            "rs",
            "struct Config {\n    max_interval: u32,\n    color: bool,\n}\n",
        );
        let body = resolve_symbol(&path, "Config").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(
            text.contains("max_interval"),
            "struct body should contain fields"
        );
    }

    #[test]
    fn resolve_c_function_with_typedef_return() {
        let path = write_temp_file(
            "c",
            "typedef int t_option;\n\nt_option ft_color_parse_hex(const char *s) {\n    return 0;\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"ft_color_parse_hex".to_string()),
            "should find function name, not return type. found: {:?}",
            symbols
        );

        let body = resolve_symbol(&path, "ft_color_parse_hex").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("return 0"));
    }

    #[test]
    fn resolve_scala_class_and_method() {
        let path = write_temp_file(
            "scala",
            r#"
case class VerificationProfile(requiredFor: Set[String]):

  def requiresVerification(level: String): Boolean =
    requiredFor.contains(level)

object VerificationProfile:

  val disabled: VerificationProfile = VerificationProfile(Set.empty)
"#,
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(symbols.contains(&"VerificationProfile".to_string()));
        assert!(symbols.contains(&"requiresVerification".to_string()));
        assert!(symbols.contains(&"disabled".to_string()));

        let body = resolve_symbol(&path, "requiresVerification").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("requiredFor.contains"));
    }

    #[test]
    fn resolve_scala_trait_and_enum() {
        let path = write_temp_file(
            "scala",
            r#"
trait Repository:
  def findById(id: String): Option[String]

enum Priority:
  case High, Medium, Low
"#,
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(symbols.contains(&"Repository".to_string()));
        assert!(symbols.contains(&"findById".to_string()));
        assert!(symbols.contains(&"Priority".to_string()));

        let body = resolve_symbol(&path, "Priority").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("High"));
    }

    #[test]
    fn resolve_zig_function() {
        let path = write_temp_file(
            "zig",
            "const std = @import(\"std\");\n\npub fn add(a: i32, b: i32) i32 {\n    return a + b;\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            !symbols.is_empty(),
            "zig should find at least one symbol, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_swift_function() {
        let path = write_temp_file(
            "swift",
            "func greet(name: String) -> String {\n    return \"Hello \\(name)\"\n}\n\nstruct Config {\n    var maxRetries: Int\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"greet".to_string()),
            "swift should find greet, got: {:?}",
            symbols
        );
        assert!(
            symbols.contains(&"Config".to_string()),
            "swift should find Config, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_php_function() {
        let path = write_temp_file(
            "php",
            "<?php\nfunction authenticate($email, $pass) {\n    return true;\n}\n\nclass User {\n    public function getName() { return $this->name; }\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"authenticate".to_string()),
            "php should find authenticate, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_go_function() {
        let path = write_temp_file(
            "go",
            "package main\n\nfunc Add(a int, b int) int {\n    return a + b\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"Add".to_string()),
            "go should find Add, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_bash_function() {
        let path = write_temp_file(
            "sh",
            "#!/bin/bash\n\nsetup() {\n    echo \"setting up\"\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"setup".to_string()),
            "bash should find setup, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_lua_function() {
        let path = write_temp_file(
            "lua",
            "function greet(name)\n    print(\"Hello \" .. name)\nend\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"greet".to_string()),
            "lua should find greet, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_julia_function() {
        let path = write_temp_file(
            "jl",
            "function fibonacci(n)\n    n <= 1 ? n : fibonacci(n-1) + fibonacci(n-2)\nend\n\nstruct Config\n    max_interval::Int\nend\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"fibonacci".to_string()),
            "julia should find fibonacci, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_csharp_class() {
        let path = write_temp_file(
            "cs",
            "public class UserService {\n    public void Authenticate(string email) {\n    }\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"UserService".to_string()),
            "c# should find UserService, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_haskell_function() {
        let path = write_temp_file(
            "hs",
            "module Main where\n\nfib :: Int -> Int\nfib 0 = 0\nfib 1 = 1\nfib n = fib (n-1) + fib (n-2)\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            !symbols.is_empty(),
            "haskell should find at least one symbol, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_odin_procedure() {
        let path = write_temp_file(
            "odin",
            "package main\n\nadd :: proc(a, b: int) -> int {\n    return a + b\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            !symbols.is_empty(),
            "odin should find at least one symbol, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_dart_class() {
        let path = write_temp_file(
            "dart",
            "class UserService {\n  void authenticate(String email) {}\n}\n",
        );
        let symbols = list_symbols(&path).unwrap();
        assert!(
            symbols.contains(&"UserService".to_string()),
            "dart should find class_declaration, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_ocaml_let_binding() {
        let path = write_temp_file("ml", "let add x y = x + y\n\nlet multiply x y = x * y\n");
        let symbols = list_symbols(&path).unwrap();
        assert!(
            !symbols.is_empty(),
            "ocaml should find let bindings via value_name, got: {:?}",
            symbols
        );
    }

    #[test]
    fn resolve_python_function() {
        let path = write_temp_file(
            "py",
            r#"
def greet(name):
    print(f"Hello {name}")

class Greeter:
    pass
"#,
        );
        let body = resolve_symbol(&path, "greet").unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn resolve_range_extracts_correct_lines() {
        let path = write_temp_file(
            "css",
            "/* line 1 */\n/* line 2 */\n/* line 3 */\n/* line 4 */\n/* line 5 */\n",
        );
        let body = resolve_range(&path, 2, 4).unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert!(
            text.contains("line 2"),
            "should contain line 2, got: {text}"
        );
        assert!(
            text.contains("line 4"),
            "should contain line 4, got: {text}"
        );
        assert!(!text.contains("line 1"), "should not contain line 1");
        assert!(!text.contains("line 5"), "should not contain line 5");
    }

    #[test]
    fn hash_range_is_stable() {
        let path = write_temp_file("css", "a {}\nb {}\nc {}\n");
        let h1 = hash_range(&path, 1, 2).unwrap();
        let h2 = hash_range(&path, 1, 2).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn resolve_range_out_of_bounds() {
        let path = write_temp_file("css", "a {}\nb {}\n");
        let err = resolve_range(&path, 1, 10).unwrap_err();
        assert!(matches!(err, SymbolError::Io { .. }));
    }

    #[test]
    fn resolve_range_single_line() {
        let path = write_temp_file("css", "line1\nline2\nline3\n");
        let body = resolve_range(&path, 2, 2).unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        assert_eq!(text, "line2");
    }
}
