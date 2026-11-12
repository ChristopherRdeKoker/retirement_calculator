import init from "./retire-calc.js";

init({ module_or_path: new URL("./retire-calc_bg.wasm", import.meta.url) }).catch(() => {
  const app = document.getElementById("app");
  if (app) {
    app.textContent = "The calculator could not load. Please refresh the page or try a newer browser.";
    app.classList.add("load-error");
  }
});
