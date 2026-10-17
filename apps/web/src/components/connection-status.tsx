import { useSyncExternalStore } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
type Notice = { unavailable: boolean; text: string };
let current: Notice = { unavailable: false, text: "" };
const listeners = new Set<() => void>();
const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};
export function setConnectionNotice(next: Notice) {
  if (current.unavailable === next.unavailable && current.text === next.text)
    return;
  current = next;
  listeners.forEach((listener) => listener());
}
export function ConnectionStatus() {
  const notice = useSyncExternalStore(subscribe, () => current);
  const reduce = useReducedMotion();
  return (
    <div id="model-health-notice" role="status" aria-live="polite">
      <AnimatePresence initial={false}>
        {notice.unavailable && (
          <motion.div
            key="connection"
            className="connection-notice"
            initial={reduce ? false : { opacity: 0, height: 0, y: 4 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: reduce ? 0 : 4 }}
            transition={{ duration: reduce ? 0 : 0.22 }}
          >
            <div className="connection-notice-inner">
              <span className="connection-notice-icon" aria-hidden="true">
                <span />
              </span>
              <span className="connection-notice-copy">
                <strong>
                  {document.documentElement.lang === "es"
                    ? "Por ahora no está disponible"
                    : "Temporarily unavailable"}
                </strong>
                <span>{notice.text}</span>
              </span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
