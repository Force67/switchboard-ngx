import { Component } from "solid-js";

interface Props {
  type: 'vision' | 'tools' | 'agent' | 'image' | 'reasoning';
  disabled?: boolean;
}

const titles: Record<Props['type'], string> = {
  vision: "Vision capable",
  tools: "Tool use / Function calling",
  agent: "Agentic capabilities",
  image: "Image generation",
  reasoning: "Extended reasoning",
};

const Badge: Component<Props> = (props) => {
  const getIcon = () => {
    switch (props.type) {
      case 'vision':
        return (
          <svg viewBox="0 0 24 24">
            <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        );
      case 'tools':
        return (
          <svg viewBox="0 0 24 24">
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
          </svg>
        );
      case 'agent':
        return (
          <svg viewBox="0 0 24 24">
            <path d="M12 8V4H8" />
            <rect x="4" y="8" width="16" height="12" rx="2" />
            <path d="M2 14h2M20 14h2M15 13v2M9 13v2" />
          </svg>
        );
      case 'image':
        return (
          <svg viewBox="0 0 24 24">
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <path d="M21 15l-5-5L5 21" />
          </svg>
        );
      case 'reasoning':
        return (
          <svg viewBox="0 0 24 24">
            <path d="M12 2a8 8 0 0 0-8 8c0 2.8 1.5 5.3 3.7 6.7L7 22l5-3 5 3-.7-5.3A8 8 0 0 0 12 2z" />
            <path d="M9 10h.01M15 10h.01M9.5 15a3.5 3.5 0 0 0 5 0" />
          </svg>
        );
    }
  };

  return (
    <span
      class={`capability-badge ${props.type} ${props.disabled ? 'disabled' : ''}`}
      title={titles[props.type]}
    >
      {getIcon()}
    </span>
  );
};

export default Badge;
