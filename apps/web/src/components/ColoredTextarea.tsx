import { createSignal, createEffect, onMount, splitProps, Component, For } from "solid-js";
import { Accessor, Setter } from "solid-js";

interface Props {
  value: Accessor<string>;
  onInput: Setter<string>;
  onKeyDown?: (e: KeyboardEvent) => void;
  onInputCustom?: (e: InputEvent) => void;
  placeholder?: string;
  style?: any;
  ref?: HTMLTextAreaElement;
}

const ColoredTextarea: Component<Props> = (props) => {
  const [local, others] = splitProps(props, ["value", "onInput", "onKeyDown", "onInputCustom", "placeholder", "style", "ref"]);
  let textareaRef: HTMLTextAreaElement;
  let overlayRef: HTMLDivElement;

  const updateOverlay = () => {
    if (!textareaRef || !overlayRef) return;

    const text = local.value();

    // Create HTML with colored @mentions
    const atRegex = /@(\w+)/g;
    const parts = [];
    let lastIndex = 0;
    let match;

    while ((match = atRegex.exec(text)) !== null) {
      const regularText = text.substring(lastIndex, match.index);
      if (regularText) {
        parts.push(`<span style="color: #fff;">${regularText}</span>`);
      }
      parts.push(`<span style="color: #0066cc; font-weight: 500;">${match[0]}</span>`);
      lastIndex = match.index + match[0].length;
    }

    const remainingText = text.substring(lastIndex);
    if (remainingText) {
      parts.push(`<span style="color: #fff;">${remainingText}</span>`);
    }

    const displayText = parts.length > 0 ? parts.join('') : `<span style="color: #999;">${local.placeholder || ''}</span>`;
    overlayRef.innerHTML = displayText;

    // Copy textarea styles to overlay
    const computedStyle = window.getComputedStyle(textareaRef);
    overlayRef.style.fontSize = computedStyle.fontSize;
    overlayRef.style.fontFamily = computedStyle.fontFamily;
    overlayRef.style.lineHeight = computedStyle.lineHeight;
    overlayRef.style.padding = computedStyle.padding;
    overlayRef.style.border = computedStyle.border;
    overlayRef.style.borderRadius = computedStyle.borderRadius;
    overlayRef.style.width = computedStyle.width;
    overlayRef.style.height = computedStyle.height;
    overlayRef.style.whiteSpace = 'pre-wrap';
    overlayRef.style.wordWrap = 'break-word';
    overlayRef.style.overflow = 'hidden';
    overlayRef.style.pointerEvents = 'none';
    overlayRef.style.position = 'absolute';
    overlayRef.style.top = '0';
    overlayRef.style.left = '0';
    overlayRef.style.background = 'transparent';
    overlayRef.style.zIndex = '1';
    overlayRef.style.color = 'inherit';

    textareaRef.style.background = 'transparent';
    textareaRef.style.zIndex = '2';
    textareaRef.style.position = 'relative';
    textareaRef.style.color = 'transparent';
    textareaRef.style.caretColor = '#fff';
  };

  createEffect(updateOverlay);

  onMount(() => {
    updateOverlay();

    // Handle scroll events
    const handleScroll = () => {
      if (overlayRef) {
        overlayRef.scrollTop = textareaRef.scrollTop;
        overlayRef.scrollLeft = textareaRef.scrollLeft;
      }
    };

    textareaRef.addEventListener('scroll', handleScroll);

    return () => {
      textareaRef.removeEventListener('scroll', handleScroll);
    };
  });

  const handleInput = (e: InputEvent) => {
    const textarea = e.currentTarget as HTMLTextAreaElement;
    local.onInput(textarea.value);

    if (local.onInputCustom) {
      local.onInputCustom(e);
    }
  };

  return (
    <div style={`position: relative; ${local.style ? Object.entries(local.style).map(([k, v]) => `${k}: ${v}`).join('; ') : ''}`}>
      <div
        ref={overlayRef!}
        style={{
          color: 'transparent',
          caretColor: 'transparent',
        }}
      />
      <textarea
        ref={textareaRef!}
        value={local.value()}
        onInput={handleInput}
        onKeyDown={local.onKeyDown}
        placeholder=""
        style={{
          background: 'transparent',
          resize: 'none',
          width: "100%",
          ...local.style,
        }}
        {...others}
      />
    </div>
  );
};

export default ColoredTextarea;