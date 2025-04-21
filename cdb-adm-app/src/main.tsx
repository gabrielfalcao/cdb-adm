import React from "react";
import ReactDOM from "react-dom/client";
import "normalize.css/normalize.css";
import "bootstrap/dist/css/bootstrap.min.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import "@blueprintjs/table/lib/css/table.css";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
  <App />
  </React.StrictMode>,
);
