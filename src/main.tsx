import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { TerminalWindow } from "./views/TerminalWindow";

const isTerminalWindow = window.location.hash === "#terminal";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isTerminalWindow ? <TerminalWindow /> : <App />}
  </React.StrictMode>,
);
