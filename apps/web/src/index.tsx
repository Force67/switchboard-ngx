import { render } from "solid-js/web";
import App from "./App";

const root = document.getElementById("root");
if (!root) {
  throw new Error("#root not found in index.html");
}

render(() => <App />, root);
