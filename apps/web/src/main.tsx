import "./styles.css";
import "./styles/identity.css";
import { createRoot } from "react-dom/client";
import { Voice } from "./live/Voice";
import { authenticate } from "./login";
document.documentElement.dataset.theme =
  localStorage.getItem("fenix.theme") === "dark" ? "dark" : "light";
document.documentElement.lang =
  localStorage.getItem("fenix.language") === "en" ? "en" : "es";
void authenticate()
  .then(async (authenticated) => {
    if (!authenticated) return;
    createRoot(document.getElementById("voice-root")!).render(<Voice />);
    await import("./controller");
  })
  .catch(() => {
    document.getElementById("app")!.textContent =
      "No se pudo conectar. Recarga la página. / Unable to connect. Reload the page.";
  });
