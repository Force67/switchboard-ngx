import { Component } from "solid-js";

interface Props {
  type: 'vision' | 'tools' | 'agent' | 'image' | 'reasoning';
  disabled?: boolean;
}

const icons = {
  vision: <svg viewBox="0 0 24 24"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>,
  tools: <svg viewBox="0 0 24 24"><path d="M22.7 19l-9.1-9.1c.9-2.3.4-5-1.5-6.9-2-2-5-2.4-7.4-1.3L9 6 6 9 1.6 4.7C.4 7.1.9 10.1 2.9 12.1c1.9 1.9 4.6 2.4 6.9 1.5l9.1 9.1c.4.4 1 .4 1.4 0l2.3-2.3c.5-.4.5-1.1.1-1.4z"/></svg>,
  agent: <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>,
  image: <svg viewBox="0 0 24 24"><path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/></svg>,
  reasoning: <svg viewBox="0 0 24 24"><path d="M12 2c-1.1 0-2 .9-2 2 0 .74.4 1.38 1 1.72v1.78l-2.5 2.5c-.39.39-.39 1.02 0 1.41l1.17 1.17c.2.2.45.29.71.29s.51-.1.71-.29L12 9.42l1.91 1.91c.2.2.45.29.71.29s.51-.1.71-.29L16.5 10.16c.39-.39.39-1.02 0-1.41L14 6.25V4.72c.6-.34 1-.98 1-1.72 0-1.1-.9-2-2-2zm0 2c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zM9.5 14.5c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5 1.5-.67 1.5-1.5-.67-1.5-1.5-1.5zm5 0c-.83 0-1.5.67-1.5 1.5s.67 1.5 1.5 1.5 1.5-.67 1.5-1.5-.67-1.5-1.5-1.5z"/></svg>,
};

const titles = {
  vision: "Vision/See",
  tools: "Tools/Structured",
  agent: "Agents/Reasoning",
  image: "Image generation",
  reasoning: "Reasoning",
};

const Badge: Component<Props> = (props) => {
  const getSimpleIcon = () => {
    switch (props.type) {
      case 'vision': return '👁';
      case 'tools': return '🔧';
      case 'agent': return '🤖';
      case 'image': return '🖼';
      case 'reasoning': return '🧠';
      default: return '❓';
    }
  };

  // Use emojis for now since they don't have the SVG disappearing issue
  return (
    <div
      class={`badge ${props.disabled ? 'dim' : ''} ${props.type}`}
      title={titles[props.type]}
      style={{
        width: "24px",
        height: "24px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: "14px",
        position: "relative",
        zIndex: 10,
        color: props.disabled ? "#999" : "#666"
      }}
    >
      {getSimpleIcon()}
    </div>
  );
};

export default Badge;