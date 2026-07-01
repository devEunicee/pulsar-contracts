import React from "react";
import { createRoot } from "react-dom/client";
import { AppRouter } from "./components/Router";
import { ToastContainer } from "./components/Toast";
import { ThemeProvider } from "./theme";
import "../../theme/tokens.css";
import "./theme/theme.css";

createRoot(document.getElementById("root")!).render(
  <ThemeProvider>
    <AppRouter />
    <ToastContainer />
  </ThemeProvider>
);
