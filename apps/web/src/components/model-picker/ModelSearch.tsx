import { Component } from "solid-js";

interface Props {
  query: string;
  onInput: (value: string) => void;
}

const ModelSearch: Component<Props> = (props) => {
  return (
    <div class="search">
      <svg viewBox="0 0 24 24">
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        type="text"
        placeholder="Search models..."
        value={props.query}
        onInput={(e) => props.onInput(e.currentTarget.value)}
      />
    </div>
  );
};

export default ModelSearch;