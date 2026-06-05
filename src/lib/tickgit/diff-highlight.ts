import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import css from "highlight.js/lib/languages/css";
import go from "highlight.js/lib/languages/go";
import java from "highlight.js/lib/languages/java";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import plaintext from "highlight.js/lib/languages/plaintext";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("css", css);
hljs.registerLanguage("go", go);
hljs.registerLanguage("java", java);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("json", json);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("plaintext", plaintext);
hljs.registerLanguage("python", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("yaml", yaml);

const EXTENSION_LANGUAGE_MAP: Record<string, string> = {
  bash: "bash",
  cjs: "javascript",
  css: "css",
  go: "go",
  htm: "xml",
  html: "xml",
  java: "java",
  js: "javascript",
  json: "json",
  jsx: "javascript",
  md: "markdown",
  mjs: "javascript",
  py: "python",
  rs: "rust",
  sh: "bash",
  svelte: "xml",
  toml: "plaintext",
  ts: "typescript",
  tsx: "typescript",
  xml: "xml",
  yaml: "yaml",
  yml: "yaml",
};

export function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export function getDiffHighlightLanguage(filePath: string | null | undefined) {
  const extension = filePath
    ?.split("?")[0]
    ?.split("#")[0]
    ?.split(".")
    .pop()
    ?.toLocaleLowerCase();

  if (!extension || extension === filePath?.toLocaleLowerCase()) {
    return null;
  }

  return EXTENSION_LANGUAGE_MAP[extension] ?? null;
}

export function highlightDiffContent(
  content: string,
  filePath: string | null | undefined,
) {
  const language = getDiffHighlightLanguage(filePath);

  if (!language || !hljs.getLanguage(language)) {
    return escapeHtml(content || " ");
  }

  return hljs.highlight(content || " ", {
    language,
    ignoreIllegals: true,
  }).value;
}
