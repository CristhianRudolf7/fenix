import { Icon } from "../components/icons";
import "./provider-icons.css";

const brands = new Set([
  "google",
  "slack",
  "github",
  "telegram",
  "whatsapp",
  "postgresql",
  "sqlite",
]);
const capabilities: Record<string, string> = {
  email: "mail",
  web: "search",
  pdf: "pdf",
  office: "office",
};

/** Local assets keep account cards recognizable without third-party requests. */
export function ProviderIcon({ provider }: { provider: string }) {
  return (
    <span
      className={`provider-icon provider-icon--${provider}`}
      aria-hidden="true"
    >
      {provider === "microsoft" ? (
        <svg viewBox="0 0 24 24" focusable="false">
          <path fill="#f25022" d="M1 1h10v10H1z" />
          <path fill="#7fba00" d="M13 1h10v10H13z" />
          <path fill="#00a4ef" d="M1 13h10v10H1z" />
          <path fill="#ffb900" d="M13 13h10v10H13z" />
        </svg>
      ) : brands.has(provider) ? (
        <img
          src={`/assets/integrations/${provider}.svg`}
          alt=""
          width="28"
          height="28"
          draggable={false}
        />
      ) : (
        <Icon name={capabilities[provider] || "integrations"} />
      )}
    </span>
  );
}
