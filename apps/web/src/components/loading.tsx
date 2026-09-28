import "./loading.css";
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
      aria-label={text}
    >
      <div className="interface-skeleton" aria-hidden="true">
        <span className="skeleton-heading" />
        <span className="skeleton-subheading" />
        <div className="skeleton-cards">
          {[0, 1, 2].map((index) => (
            <div className="skeleton-card" key={index}>
              <span className="skeleton-icon" />
              <span />
              <span />
              <span />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
