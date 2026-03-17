import * as fs from "fs";
import * as path from "path";

// eslint-disable-next-line @typescript-eslint/no-require-imports
const TreeSitter = require("tree-sitter");

type Language = unknown;

interface LanguageLoader {
  extensions: string[];
  load: () => Language;
}

const LANGUAGE_LOADERS: LanguageLoader[] = [
  {
    extensions: [".ts", ".tsx"],
    load: () => {
      // eslint-disable-next-line @typescript-eslint/no-require-imports
      const ts = require("tree-sitter-typescript");
      return ts.typescript;
    },
  },
  {
    extensions: [".js", ".jsx", ".mjs", ".cjs"],
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    load: () => require("tree-sitter-javascript"),
  },
  {
    extensions: [".py"],
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    load: () => require("tree-sitter-python"),
  },
  {
    extensions: [".go"],
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    load: () => require("tree-sitter-go"),
  },
];

const SYMBOL_NODE_TYPES: Record<string, string[]> = {
  ".ts": ["function_declaration", "class_declaration", "type_alias_declaration", "interface_declaration", "variable_declarator", "method_definition"],
  ".tsx": ["function_declaration", "class_declaration", "type_alias_declaration", "interface_declaration", "variable_declarator", "method_definition"],
  ".js": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
  ".jsx": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
  ".mjs": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
  ".cjs": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
  ".py": ["function_definition", "class_definition"],
  ".go": ["function_declaration", "method_declaration", "type_spec"],
};

interface TreeNode {
  type: string;
  text: string;
  children: TreeNode[];
}

function extractNamesFromTree(node: TreeNode, targetTypes: string[], results: string[] = []): string[] {
  if (targetTypes.includes(node.type)) {
    for (const child of node.children) {
      if (child.type === "identifier" || child.type === "type_identifier") {
        results.push(child.text);
        break;
      }
    }
  }
  for (const child of node.children) {
    extractNamesFromTree(child, targetTypes, results);
  }
  return results;
}

export function extractSymbols(filePath: string): string[] {
  const ext = path.extname(filePath).toLowerCase();
  const loader = LANGUAGE_LOADERS.find((l) => l.extensions.includes(ext));
  if (!loader) return [];

  const targetTypes = SYMBOL_NODE_TYPES[ext] ?? [];
  if (!targetTypes.length) return [];

  const source = fs.readFileSync(filePath, "utf8");
  const parser = new TreeSitter();
  parser.setLanguage(loader.load());
  const tree = parser.parse(source);
  return extractNamesFromTree(tree.rootNode as TreeNode, targetTypes);
}

export function supportsSymbolExtraction(filePath: string): boolean {
  const ext = path.extname(filePath).toLowerCase();
  return LANGUAGE_LOADERS.some((l) => l.extensions.includes(ext));
}
