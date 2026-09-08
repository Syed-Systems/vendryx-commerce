import React from "react"
import ReactDOM from "react-dom/client"
import { App } from "./App"
import "../modules/styles/tokens.css"

const root = document.getElementById("root")

if (!root) {
  throw new Error("Mahal demo root was not found")
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
