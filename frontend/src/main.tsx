/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import "./theme/tokens.css";
import "./theme/app.css";

render(() => <App />, document.getElementById("root")!);
