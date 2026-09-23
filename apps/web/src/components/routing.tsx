import { useEffect, type ReactNode } from "react";
import {
  BrowserRouter,
  NavLink,
  useLocation,
  useNavigate,
  type NavigateFunction,
} from "react-router";

export const pages = [
  "chat",
  "projects",
  "memory",
  "skills",
  "design",
  "documents",
  "browser",
  "integrations",
  "schedules",
  "activity",
  "settings",
] as const;
export type Page = (typeof pages)[number];
export function currentPage(path = window.location.pathname): Page {
  const value = path.replace(/^\/+|\/+$/g, "");
  return pages.includes(value as Page) ? (value as Page) : "chat";
}
let navigate: NavigateFunction | undefined;
let listener: ((page: Page) => void) | undefined;
export function onPageChange(callback: (page: Page) => void) {
  listener = callback;
}
export function navigateTo(page: Page) {
  void navigate?.(`/${page}`);
}
function LocationObserver() {
  const location = useLocation();
  navigate = useNavigate();
  useEffect(() => {
    const page = currentPage(location.pathname);
    if (location.pathname !== `/${page}`) {
      void navigate?.(`/${page}`, { replace: true });
      return;
    }
    queueMicrotask(() => listener?.(page));
  }, [location.pathname]);
  return null;
}
export function ApplicationRouter({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter>
      <LocationObserver />
      {children}
    </BrowserRouter>
  );
}
export { NavLink };
