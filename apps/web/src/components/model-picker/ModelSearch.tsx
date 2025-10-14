import { Component } from "solid-js";

interface Props {
  query: string;
  onInput: (value: string) => void;
  inputRef?: (el: HTMLInputElement) => void;
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
        ref={props.inputRef}
        onInput={(e) => props.onInput(e.currentTarget.value)}
      />
    </div>
  );
};

export default ModelSearch;
