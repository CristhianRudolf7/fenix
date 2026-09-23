/** Shared, local vector icons for React and the existing DOM controllers. */
const paths: Record<string, string> = {
  documents: "M5 3h10l4 4v14H5V3Zm10 0v5h4M8 12h8M8 16h6",
  menu: "M3 5h18v14H3V5Zm5 0v14M12 9h5M12 13h5",
  chat: "M21 11.5a8.5 8.5 0 0 1-8.5 8.5H4l-2 2v-9.5A8.5 8.5 0 0 1 10.5 4H13a8 8 0 0 1 8 7.5ZM7 10h9M7 14h6",
  projects:
    "M3 7V5a2 2 0 0 1 2-2h5l3 3h6a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z",
  browser:
    "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM3 12h18M12 3c5 5 5 13 0 18-5-5-5-13 0-18Z",
  integrations: "M8 3v5M16 3v5M6 8h12v4a6 6 0 0 1-12 0V8ZM12 18v4",
  memory:
    "M12 18V5a3 3 0 0 0-5.8-1A4 4 0 0 0 3 10a4 4 0 0 0 1 7 4 4 0 0 0 8 1Zm0 0V5a3 3 0 0 1 5.8-1A4 4 0 0 1 21 10a4 4 0 0 1-1 7 4 4 0 0 1-8 1ZM7 8c0 2 1 3 3 3M17 8c0 2-1 3-3 3M7 17v-3M17 17v-3",
  skills: "M4 3h6v6H4zM14 3h6v6h-6zM4 13h6v6H4zM17 13v8M13 17h8",
  design:
    "M3 3h8v8H3zM18 3l4 8h-8l4-8ZM11 18a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM15 15h6v6h-6z",
  schedules:
    "M7 2v4M17 2v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2ZM7 14h2M13 14h4M7 18h2",
  activity: "m3 6 2 2 4-4M12 6h9m-18 9 2 2 4-4M12 15h9M12 20h6",
  settings:
    "M3 6h4M11 6h10M3 12h10M17 12h4M3 18h4M11 18h10M11 6a2 2 0 1 1-4 0 2 2 0 0 1 4 0ZM17 12a2 2 0 1 1-4 0 2 2 0 0 1 4 0ZM11 18a2 2 0 1 1-4 0 2 2 0 0 1 4 0Z",
  mail: "M4 5h16a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Zm-1 2 9 6 9-6",
  search: "M16 10a6 6 0 1 1-12 0 6 6 0 0 1 12 0Zm-2 5 7 7",
  pdf: "M5 2h9l5 5v15H5V2Zm9 0v6h5M8 12h2a2 2 0 0 1 0 4H8m0-4v7M14 13h2M14 16h2M14 19h2",
  office:
    "M3 3h11v14H3zM14 7h7v14H8v-4M6 7h5M6 10h5M6 13h3M17 11h1M17 14h1M12 18h6",
  terminal: "M3 4h18v16H3V4Zm4 5 3 3-3 3M13 16h4",
  condition: "m12 2 10 10-10 10L2 12 10-10ZM12 8v5M12 16h.01",
  question:
    "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM9.5 8.5a2.5 2.5 0 1 1 4 2c-1 .7-1.5 1-1.5 2.5M12 17h.01",
  notification: "M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9ZM10 21h4",
  sun: "M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0ZM12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5 19 19M5 19l1.5-1.5M17.5 6.5 19 5",
  moon: "M20 15.5A9 9 0 0 1 8.5 4 9 9 0 1 0 20 15.5Z",
  arrow: "M4 12h16m-6-6 6 6-6 6",
  send: "M12 20V4m-7 7 7-7 7 7",
  mic: "M9 5a3 3 0 0 1 6 0v7a3 3 0 0 1-6 0V5Zm-4 6v1a7 7 0 0 0 14 0v-1M12 19v3M8 22h8",
  wave: "M4 10v4M8 6v12M12 3v18M16 7v10M20 10v4",
  stop: "M6 6h12v12H6Z",
  eye: "M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
  hide: "m3 3 18 18M9 5.5c6-2 11 4 13 6.5-.8 1.2-1.8 2.5-3 3.5M6 7c-2 2-3 4-4 5 3 5 8 9 14 6M10 10a3 3 0 0 0 4 4",
  logout: "M9 4H4v16h5M10 12h12m-5-5 5 5-5 5",
  chevron: "m9 5 7 7-7 7",
};

export function iconMarkup(name: string): string {
  return `<svg class="ui-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><path d="${paths[name] || paths.skills}"/></svg>`;
}

export function Icon({ name }: { name: string }) {
  return (
    <span
      className="icon-wrap"
      dangerouslySetInnerHTML={{ __html: iconMarkup(name) }}
    />
  );
}

export const flameMarkup =
  '<svg class="flame" width="28" height="34" viewBox="0 0 32 40" fill="currentColor" aria-hidden="true" focusable="false"><path d="M25 1c2 9-3 13-11 17C7 21 5 26 8 33 1 28 1 19 8 14 15 9 21 9 25 1Z"/><path d="M29 13c1 9-3 14-11 18-4 2-5 5-3 9-7-5-6-11 1-16 6-4 10-5 13-11Z"/></svg>';

export function Flame() {
  return (
    <span
      className="flame-wrap"
      dangerouslySetInnerHTML={{ __html: flameMarkup }}
    />
  );
}
