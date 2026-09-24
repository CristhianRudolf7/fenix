import { Flame } from "./icons";
/** Indeterminate progress: decorative movement, one accessible status label. */
export function Loading({ label }: { label?: string }) {
  const text =
    label ??
    (document.documentElement.lang === "en"
      ? "Preparing your space…"
      : "Preparando tu espacio…");
  return (
    <div
      className="loading-state"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <span className="loading-emblem" aria-hidden="true">
        <Flame />
      </span>
      <span className="loading-label">{text}</span>
    </div>
  );
}
