import { Component } from "solid-js";

interface Props {
  type: 'vision' | 'tools' | 'agent' | 'image';
  disabled?: boolean;
}

const icons = {
  vision: <svg viewBox="0 0 24 24"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>,
  tools: <svg viewBox="0 0 24 24"><path d="M12 2l2.6 5.7 6.3.9-4.6 4.5 1.1 6.3L12 16.9 6.6 19.4l1.1-6.3L3 8.6l6.3-.9L12 2z"/></svg>,
  agent: <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg>,
  image: <svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>,
};

const titles = {
  vision: "Vision/See",
  tools: "Tools/Structured",
  agent: "Agents/Reasoning",
  image: "Image generation",
};

const Badge: Component<Props> = (props) => {
  return (
    <div class={`badge ${props.disabled ? 'dim' : ''}`} title={titles[props.type]}>
      {icons[props.type]}
    </div>
  );
};

export default Badge;