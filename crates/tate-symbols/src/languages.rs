use tree_sitter::Language;

pub struct LanguageConfig {
    pub language: Language,
    pub symbol_node_types: &'static [&'static str],
}

pub fn language_for_extension(ext: &str) -> Option<LanguageConfig> {
    match ext {
        "rs" => Some(LanguageConfig {
            language: tree_sitter_rust::LANGUAGE.into(),
            symbol_node_types: &[
                "function_item",
                "struct_item",
                "enum_item",
                "impl_item",
                "trait_item",
                "type_item",
                "const_item",
                "static_item",
                "macro_definition",
            ],
        }),
        "py" => Some(LanguageConfig {
            language: tree_sitter_python::LANGUAGE.into(),
            symbol_node_types: &["function_definition", "class_definition"],
        }),
        "js" | "jsx" => Some(LanguageConfig {
            language: tree_sitter_javascript::LANGUAGE.into(),
            symbol_node_types: &[
                "function_declaration",
                "class_declaration",
                "lexical_declaration",
            ],
        }),
        "ts" => Some(LanguageConfig {
            language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            symbol_node_types: &[
                "function_declaration",
                "class_declaration",
                "interface_declaration",
                "type_alias_declaration",
                "lexical_declaration",
                "enum_declaration",
            ],
        }),
        "tsx" => Some(LanguageConfig {
            language: tree_sitter_typescript::LANGUAGE_TSX.into(),
            symbol_node_types: &[
                "function_declaration",
                "class_declaration",
                "interface_declaration",
                "type_alias_declaration",
                "lexical_declaration",
                "enum_declaration",
            ],
        }),
        "go" => Some(LanguageConfig {
            language: tree_sitter_go::LANGUAGE.into(),
            symbol_node_types: &[
                "function_declaration",
                "method_declaration",
                "type_declaration",
            ],
        }),
        "java" => Some(LanguageConfig {
            language: tree_sitter_java::LANGUAGE.into(),
            symbol_node_types: &[
                "method_declaration",
                "class_declaration",
                "interface_declaration",
                "enum_declaration",
                "constructor_declaration",
            ],
        }),
        "c" | "h" => Some(LanguageConfig {
            language: tree_sitter_c::LANGUAGE.into(),
            symbol_node_types: &[
                "function_definition",
                "struct_specifier",
                "enum_specifier",
                "type_definition",
            ],
        }),
        "cpp" | "hpp" | "cc" | "cxx" | "hh" => Some(LanguageConfig {
            language: tree_sitter_cpp::LANGUAGE.into(),
            symbol_node_types: &[
                "function_definition",
                "class_specifier",
                "struct_specifier",
                "namespace_definition",
                "type_definition",
            ],
        }),
        "rb" => Some(LanguageConfig {
            language: tree_sitter_ruby::LANGUAGE.into(),
            symbol_node_types: &["method", "class", "module", "singleton_method"],
        }),
        "odin" => Some(LanguageConfig {
            language: tree_sitter_odin::LANGUAGE.into(),
            symbol_node_types: &[
                "procedure_declaration",
                "struct_declaration",
                "enum_declaration",
                "union_declaration",
                "constant_declaration",
            ],
        }),
        "dart" => Some(LanguageConfig {
            language: tree_sitter_dart::LANGUAGE.into(),
            symbol_node_types: &[
                "function_signature",
                "class_declaration",
                "method_signature",
                "enum_declaration",
            ],
        }),
        "ex" | "exs" => Some(LanguageConfig {
            language: tree_sitter_elixir::LANGUAGE.into(),
            symbol_node_types: &["call"],
        }),
        "gleam" => Some(LanguageConfig {
            language: tree_sitter_gleam::LANGUAGE.into(),
            symbol_node_types: &["function", "type_definition", "type_alias"],
        }),
        "scala" | "sc" => Some(LanguageConfig {
            language: tree_sitter_scala::LANGUAGE.into(),
            symbol_node_types: &[
                "function_definition",
                "function_declaration",
                "class_definition",
                "object_definition",
                "trait_definition",
                "enum_definition",
                "val_definition",
                "type_definition",
            ],
        }),
        "zig" => Some(LanguageConfig {
            language: tree_sitter_zig::LANGUAGE.into(),
            symbol_node_types: &[
                "function_declaration",
                "variable_declaration",
                "test_declaration",
                "struct_declaration",
                "enum_declaration",
                "union_declaration",
            ],
        }),
        "ml" => Some(LanguageConfig {
            language: tree_sitter_ocaml::LANGUAGE_OCAML.into(),
            symbol_node_types: &["let_binding", "type_definition", "module_definition"],
        }),
        "mli" => Some(LanguageConfig {
            language: tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE.into(),
            symbol_node_types: &[
                "value_specification",
                "type_definition",
                "module_definition",
            ],
        }),
        "clj" | "cljs" | "cljc" | "edn" => None,
        "swift" => Some(LanguageConfig {
            language: tree_sitter_swift::LANGUAGE.into(),
            symbol_node_types: &[
                "function_declaration",
                "class_declaration",
                "struct_declaration",
                "enum_declaration",
                "protocol_declaration",
            ],
        }),
        "hs" => Some(LanguageConfig {
            language: tree_sitter_haskell::LANGUAGE.into(),
            symbol_node_types: &[
                "function",
                "data",
                "type_synomym",
                "class_decl",
                "instance_decl",
            ],
        }),
        "lua" => Some(LanguageConfig {
            language: tree_sitter_lua::LANGUAGE.into(),
            symbol_node_types: &["function_declaration", "function_definition"],
        }),
        "sh" | "bash" => Some(LanguageConfig {
            language: tree_sitter_bash::LANGUAGE.into(),
            symbol_node_types: &["function_definition"],
        }),
        "php" => Some(LanguageConfig {
            language: tree_sitter_php::LANGUAGE_PHP.into(),
            symbol_node_types: &[
                "function_definition",
                "class_declaration",
                "method_declaration",
                "interface_declaration",
                "trait_declaration",
            ],
        }),
        "cs" => Some(LanguageConfig {
            language: tree_sitter_c_sharp::LANGUAGE.into(),
            symbol_node_types: &[
                "method_declaration",
                "class_declaration",
                "interface_declaration",
                "struct_declaration",
                "enum_declaration",
            ],
        }),
        "r" | "R" => Some(LanguageConfig {
            language: tree_sitter_r::LANGUAGE.into(),
            symbol_node_types: &["function_definition"],
        }),
        "jl" => Some(LanguageConfig {
            language: tree_sitter_julia::LANGUAGE.into(),
            symbol_node_types: &[
                "function_definition",
                "struct_definition",
                "macro_definition",
                "module_definition",
            ],
        }),
        "sql" => None,
        _ => None,
    }
}

pub fn is_supported(ext: &str) -> bool {
    language_for_extension(ext).is_some() || matches!(ext, "sql")
}

pub fn supports_symbols(ext: &str) -> bool {
    language_for_extension(ext).is_some()
}
