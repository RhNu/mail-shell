import { render } from "solid-js/web";
import { HashRouter, Route } from "@solidjs/router";
import App from "./App";
import "./index.css";

render(
  () => (
    <HashRouter>
      <Route path="/" component={App} />
    </HashRouter>
  ),
  document.getElementById("root") as HTMLElement
);

