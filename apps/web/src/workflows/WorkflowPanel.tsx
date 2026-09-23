import { useEffect, useState, useSyncExternalStore } from "react";
import { api } from "../live/api";
import { snapshot, subscribe } from "../workspace/store";
import { FlowView } from "./FlowView";
import type { Flow } from "./types";
export function WorkflowPanel() {
  const state = useSyncExternalStore(subscribe, snapshot),
    tab = state.tabs.find((t) => t.key === state.active);
  const id = state.visible && tab?.kind === "workflow" ? tab.id : undefined;
  const [flow, setFlow] = useState<Flow | null>(null),
    [error, setError] = useState("");
  useEffect(() => {
    setFlow(null);
    setError("");
    if (!id) return;
    const controller = new AbortController();
    void api<Flow>(`/workflows/${id}`, undefined, "GET", controller.signal)
      .then(setFlow)
      .catch((e) => {
        if (!controller.signal.aborted) setError(String(e));
      });
    return () => controller.abort();
  }, [id, tab?.refresh, state.session]);
  if (!id) return null;
  return (
    <aside
      className="design-panel workflow-panel"
      aria-label={
        state.language === "es" ? "Vista del flujo" : "Workflow preview"
      }
    >
      {error && <p role="alert">{error}</p>}
      {flow ? (
        <FlowView flow={flow} language={state.language} compact />
      ) : (
        <div className="flow-loading" aria-busy="true">
          ◌
        </div>
      )}
    </aside>
  );
}
