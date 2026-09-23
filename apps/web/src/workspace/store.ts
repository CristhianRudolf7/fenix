export type Target = {
  kind: "file" | "design" | "library" | "design-choice" | "workflow";
  id?: string;
  path?: string;
  title?: string;
  project_id?: string | null;
  revision?: number;
  page?: number;
  chunk?: string;
  template?: boolean;
  refresh?: number;
};
export type Tab = Target & { key: string };
type Adapter = { show: (tab: Tab) => void; hide: () => void };
const adapters = new Map<string, Adapter>();
let state: {
  tabs: Tab[];
  active: string | null;
  visible: boolean;
  explorer: boolean;
  language: string;
  session: string;
  project: string | null;
} = {
  tabs: [],
  active: null,
  visible: false,
  explorer: false,
  language: "es",
  session: "",
  project: null,
};
const listeners = new Set<() => void>();
export const snapshot = () => state;
export const subscribe = (fn: () => void) => {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
};
function update(change: Partial<typeof state>) {
  state = { ...state, ...change };
  document.documentElement.classList.toggle("workspace-open", state.visible);
  listeners.forEach((fn) => fn());
}
export function registerAdapter(kind: Target["kind"], adapter: Adapter) {
  adapters.set(kind, adapter);
}
export function configureWorkspace(
  language: string,
  session: string,
  project: string | null,
) {
  if (state.session && state.session !== session) {
    adapters.forEach((a) => a.hide());
    update({ tabs: [], active: null, visible: false, explorer: false });
  }
  update({ language, session, project });
}
export const targetKey = (t: Target) =>
  `${t.kind}:${t.id || `${t.project_id || "general"}:${t.path || ""}`}:${t.template ? "template" : ""}`;
export function openTarget(target: Target) {
  if (
    !["file", "design", "library", "design-choice", "workflow"].includes(
      target.kind,
    ) ||
    !(target.id || target.path)
  )
    return;
  const key = targetKey(target);
  const old = state.tabs.find((t) => t.key === key);
  const tab = {
    ...old,
    ...target,
    page: target.page ?? old?.page,
    key,
    refresh: Date.now(),
  };
  const tabs = old
    ? state.tabs.map((t) => (t.key === key ? tab : t))
    : [...state.tabs, tab];
  adapters.forEach((a, k) => {
    if (k !== tab.kind) a.hide();
  });
  update({ tabs, active: key, visible: true, explorer: false });
  adapters.get(tab.kind)?.show(tab);
}
export function updateTab(key: string, patch: Partial<Target>) {
  update({
    tabs: state.tabs.map((t) => (t.key === key ? { ...t, ...patch } : t)),
  });
}
export function activateTab(key: string) {
  const tab = state.tabs.find((t) => t.key === key);
  if (tab) openTarget(tab);
}
export function hideWorkspace() {
  adapters.forEach((a) => a.hide());
  update({ visible: false, explorer: false });
}
export function closeTab(key: string) {
  const index = state.tabs.findIndex((t) => t.key === key);
  const tabs = state.tabs.filter((t) => t.key !== key);
  update({ tabs });
  if (key === state.active) {
    const next = tabs[Math.min(index, tabs.length - 1)];
    if (next) activateTab(next.key);
    else {
      hideWorkspace();
      update({ active: null });
    }
  }
}
export function showExplorer() {
  update({ explorer: true });
}
export function closeExplorer() {
  update({ explorer: false });
}
export function activeTarget() {
  return state.tabs.find((t) => t.key === state.active);
}
export const tr = (en: string, es: string) =>
  state.language === "es" ? es : en;
