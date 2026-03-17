"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractSymbols = extractSymbols;
exports.supportsSymbolExtraction = supportsSymbolExtraction;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
// eslint-disable-next-line @typescript-eslint/no-require-imports
const TreeSitter = require("tree-sitter");
const LANGUAGE_LOADERS = [
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
const SYMBOL_NODE_TYPES = {
    ".ts": ["function_declaration", "class_declaration", "type_alias_declaration", "interface_declaration", "variable_declarator", "method_definition"],
    ".tsx": ["function_declaration", "class_declaration", "type_alias_declaration", "interface_declaration", "variable_declarator", "method_definition"],
    ".js": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
    ".jsx": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
    ".mjs": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
    ".cjs": ["function_declaration", "class_declaration", "variable_declarator", "method_definition"],
    ".py": ["function_definition", "class_definition"],
    ".go": ["function_declaration", "method_declaration", "type_spec"],
};
function extractNamesFromTree(node, targetTypes, results = []) {
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
function extractSymbols(filePath) {
    const ext = path.extname(filePath).toLowerCase();
    const loader = LANGUAGE_LOADERS.find((l) => l.extensions.includes(ext));
    if (!loader)
        return [];
    const targetTypes = SYMBOL_NODE_TYPES[ext] ?? [];
    if (!targetTypes.length)
        return [];
    const source = fs.readFileSync(filePath, "utf8");
    const parser = new TreeSitter();
    parser.setLanguage(loader.load());
    const tree = parser.parse(source);
    return extractNamesFromTree(tree.rootNode, targetTypes);
}
function supportsSymbolExtraction(filePath) {
    const ext = path.extname(filePath).toLowerCase();
    return LANGUAGE_LOADERS.some((l) => l.extensions.includes(ext));
}
//# sourceMappingURL=symbols.js.map