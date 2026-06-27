/**
 * Pulsar Theme Switcher — Issue #270
 * Reads/writes the `data-theme` attribute on <html> and persists the choice
 * to localStorage so the preference survives page reloads.
 */

const STORAGE_KEY = "pulsar-theme";
const THEMES = /** @type {const} */ (["light", "dark"]);

/**
 * Apply a theme by setting the data-theme attribute on the root element.
 * @param {"light"|"dark"} theme
 */
function applyTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem(STORAGE_KEY, theme);
}

/**
 * Return the currently active theme.
 * @returns {"light"|"dark"}
 */
function getTheme() {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark") return stored;
  // Honour OS preference when no explicit choice has been stored.
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

/** Toggle between light and dark. */
function toggleTheme() {
  applyTheme(getTheme() === "dark" ? "light" : "dark");
}

/** Initialise the theme as early as possible to avoid flash. */
function initTheme() {
  applyTheme(getTheme());
}

export { THEMES, applyTheme, getTheme, initTheme, toggleTheme };
