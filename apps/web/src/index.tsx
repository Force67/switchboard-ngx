import { render } from "solid-js/web";
import App from "./App";
import "./index.css"; // make sure this path is correct

const root = document.getElementById("root");
if (!root) {
  throw new Error("#root not found in index.html");
}

render(() => <App />, root);
