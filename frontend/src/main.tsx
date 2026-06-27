import React from "react";
import { createRoot } from "react-dom/client";
import { AppRouter } from "./components/Router";

createRoot(document.getElementById("root")!).render(<AppRouter />);
