import { createMemo } from "solid-js";
import katex from "katex";
import "katex/dist/katex.min.css";

interface Props {
  content: string;
}

export default function LatexRenderer(props: Props) {
  const renderedContent = createMemo(() => {
    const text = props.content;
    const parts: (string | { type: 'inline' | 'display'; math: string })[] = [];
    let lastIndex = 0;

    // Regex for display math: $$...$$
    const displayRegex = /\$\$([^$]+)\$\$/g;
    let match;
    while ((match = displayRegex.exec(text)) !== null) {
      // Add text before the match
      if (match.index > lastIndex) {
        parts.push(text.slice(lastIndex, match.index));
      }
      // Add the math
      parts.push({ type: 'display', math: match[1] });
      lastIndex = match.index + match[0].length;
    }

    // Add remaining text
    if (lastIndex < text.length) {
      parts.push(text.slice(lastIndex));
    }

    // Now process inline math in the text parts
    const finalParts: (string | { type: 'inline' | 'display'; math: string })[] = [];
    for (const part of parts) {
      if (typeof part === 'string') {
        let inlineLastIndex = 0;
        const inlineRegex = /\$([^$]+)\$/g;
        let inlineMatch;
        while ((inlineMatch = inlineRegex.exec(part)) !== null) {
          // Add text before the match
          if (inlineMatch.index > inlineLastIndex) {
            finalParts.push(part.slice(inlineLastIndex, inlineMatch.index));
          }
          // Add the inline math
          finalParts.push({ type: 'inline', math: inlineMatch[1] });
          inlineLastIndex = inlineMatch.index + inlineMatch[0].length;
        }
        // Add remaining text
        if (inlineLastIndex < part.length) {
          finalParts.push(part.slice(inlineLastIndex));
        }
      } else {
        finalParts.push(part);
      }
    }

    return finalParts;
  });

  return (
    <div style="white-space: pre-wrap;">
      {renderedContent().map((part, index) => {
        if (typeof part === 'string') {
          return <span key={index} innerHTML={part.replace(/\n/g, '<br>')} />;
        } else {
          try {
            const html = katex.renderToString(part.math, {
              displayMode: part.type === 'display',
              throwOnError: false,
            });
            return <span key={index} innerHTML={html} />;
          } catch (error) {
            return <span key={index}>${part.type === 'display' ? '$' : ''}${part.math}${part.type === 'display' ? '$' : ''}</span>;
          }
        }
      })}
    </div>
  );
}