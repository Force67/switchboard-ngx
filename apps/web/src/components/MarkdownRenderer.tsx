import { createMemo } from "solid-js";
import { marked, type Tokens } from "marked";
import katex from "katex";
import hljs from "highlight.js";
import "katex/dist/katex.min.css";
import "highlight.js/styles/github-dark.css";

interface Props {
  content: string;
}

// Custom renderer to handle code blocks with syntax highlighting
const renderer = new marked.Renderer();

renderer.code = function ({ text, lang }: Tokens.Code) {
  const language = lang && hljs.getLanguage(lang) ? lang : "plaintext";
  const highlighted = hljs.highlight(text, { language }).value;
  return `<pre class="hljs"><code class="language-${language}">${highlighted}</code></pre>`;
};

// Make links open in new tab
renderer.link = function ({ href, title, text }: Tokens.Link) {
  const titleAttr = title ? ` title="${title}"` : "";
  return `<a href="${href}"${titleAttr} target="_blank" rel="noopener noreferrer">${text}</a>`;
};

// Configure marked for better rendering
marked.use({
  renderer,
  breaks: true,
  gfm: true,
});

export default function MarkdownRenderer(props: Props) {
  const renderedContent = createMemo(() => {
    let text = props.content;

    // Store for LaTeX expressions - use HTML comments as placeholders (markdown preserves these)
    const mathExpressions: { id: string; tex: string; display: boolean }[] = [];
    let mathId = 0;
    const mathPlaceholder = (n: number) => `<!--MATH${n}-->`;

    // Protect code blocks first (they may contain $ signs or \[ \])
    const codeBlocks: { id: string; content: string }[] = [];
    let codeId = 0;
    const codePlaceholder = (n: number) => `<!--CODE${n}-->`;

    // Protect fenced code blocks
    text = text.replace(/```[\s\S]*?```/g, (match) => {
      const id = codePlaceholder(codeId++);
      codeBlocks.push({ id, content: match });
      return id;
    });

    // Protect inline code
    text = text.replace(/`[^`]+`/g, (match) => {
      const id = codePlaceholder(codeId++);
      codeBlocks.push({ id, content: match });
      return id;
    });

    // Extract display math: $$...$$ and \[...\]
    text = text.replace(/\$\$([\s\S]+?)\$\$/g, (_, tex) => {
      const id = mathPlaceholder(mathId++);
      mathExpressions.push({ id, tex: tex.trim(), display: true });
      return id;
    });
    text = text.replace(/\\\[([\s\S]+?)\\\]/g, (_, tex) => {
      const id = mathPlaceholder(mathId++);
      mathExpressions.push({ id, tex: tex.trim(), display: true });
      return id;
    });

    // Extract inline math: $...$ and \(...\)
    text = text.replace(/\\\(([\s\S]+?)\\\)/g, (_, tex) => {
      const id = mathPlaceholder(mathId++);
      mathExpressions.push({ id, tex: tex.trim(), display: false });
      return id;
    });
    // $...$ - but not things like "$5 and $10"
    // Only match if there's non-space content between $ signs
    text = text.replace(/\$([^\s$][^$]*?[^\s$])\$/g, (_, tex) => {
      const id = mathPlaceholder(mathId++);
      mathExpressions.push({ id, tex: tex.trim(), display: false });
      return id;
    });

    // Also match single character/short inline math like $x$, $a$
    text = text.replace(/\$([^\s$]{1,3})\$/g, (match, tex) => {
      // Skip if already processed
      if (match.includes("<!--MATH")) return match;
      const id = mathPlaceholder(mathId++);
      mathExpressions.push({ id, tex: tex.trim(), display: false });
      return id;
    });

    // Restore code blocks before markdown parsing
    for (const block of codeBlocks) {
      text = text.replace(block.id, block.content);
    }

    // Parse markdown
    let html = marked.parse(text, { async: false }) as string;

    // Render LaTeX and replace placeholders
    for (const expr of mathExpressions) {
      try {
        const rendered = katex.renderToString(expr.tex, {
          displayMode: expr.display,
          throwOnError: false,
          trust: true,
        });
        const wrapper = expr.display
          ? `<div class="math-display">${rendered}</div>`
          : `<span class="math-inline">${rendered}</span>`;
        html = html.replace(expr.id, wrapper);
      } catch (e) {
        // On error, show the original LaTeX
        const escaped = expr.tex
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;");
        const fallback = expr.display
          ? `<div class="math-display math-error">$$${escaped}$$</div>`
          : `<span class="math-inline math-error">$${escaped}$</span>`;
        html = html.replace(expr.id, fallback);
      }
    }

    return html;
  });

  return (
    <div class="markdown-content" innerHTML={renderedContent()} />
  );
}
