import { Component } from "solid-js";

interface Props {
  expanded: boolean;
  onToggle: () => void;
}

const FooterBar: Component<Props> = (props) => {
  return (
    <div class="footer">
      <div class={`showall ${props.expanded ? 'expanded' : ''}`} onClick={props.onToggle}>
        <svg class="chev" viewBox="0 0 24 24">
          <path d="M7 10l5 5 5-5z" />
        </svg>
        {props.expanded ? 'Show less' : 'Show all'}
        <div class="dot"></div>
      </div>
      <button class="filterbtn">
        <svg viewBox="0 0 24 24">
          <path d="M3 4V2h18v2H3zm0 3h14v-1H3v1zm0 3h14v-1H3v1zm0 3h14v-1H3v1zm0 3h14v-1H3v1zm0 3h14v-1H3v1z" />
        </svg>
      </button>
    </div>
  );
};

export default FooterBar;