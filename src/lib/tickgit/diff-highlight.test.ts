import { describe, expect, it } from "vitest";
import {
  escapeHtml,
  getDiffHighlightLanguage,
  highlightDiffContent,
} from "$lib/tickgit/diff-highlight";

describe("diff highlight", () => {
  it("maps common file extensions to highlight languages", () => {
    expect(getDiffHighlightLanguage("src/app.ts")).toBe("typescript");
    expect(getDiffHighlightLanguage("src/App.svelte")).toBe("xml");
    expect(getDiffHighlightLanguage("README.md")).toBe("markdown");
    expect(getDiffHighlightLanguage("script.sh")).toBe("bash");
    expect(getDiffHighlightLanguage("Cargo.toml")).toBe("plaintext");
    expect(getDiffHighlightLanguage("unknown.tickgit")).toBe(null);
  });

  it("escapes unknown language content without highlighting", () => {
    expect(highlightDiffContent("<tag>", "file.unknown")).toBe("&lt;tag&gt;");
    expect(escapeHtml(`&<>"'`)).toBe("&amp;&lt;&gt;&quot;&#39;");
  });

  it("highlights known language content", () => {
    const html = highlightDiffContent("const value = 1;", "src/app.ts");

    expect(html).toContain("hljs-keyword");
    expect(html).toContain("const");
  });
});
