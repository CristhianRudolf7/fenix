# Interface and visual identity

Fenix uses a warm light theme by default: ivory canvas (`#F7F6F2`), white surfaces, charcoal text (`#23282D`), terracotta actions (`#B84D35`) and dark teal status accents (`#28635D`). The dark theme uses charcoal-green surfaces and lighter terracotta accents. Both language and theme choices persist locally and apply to the login as well as the application.

`apps/web/src/styles/identity.css` owns shared identity and responsive composition. Existing feature styles continue to own their domain-specific layouts. Local icons in `components/icons.tsx` share a vector vocabulary between React components and DOM controllers. Editorial serif headings use system fonts; reading and controls use the existing sans-serif stack. No remote fonts, image hosts or new runtime providers are required.

The login combines local artwork with an email/password form. Its compact mobile header leaves room for the form. Password visibility is optional; authentication, throttling and session handling remain in the existing gateway. The logout action sits in the application header and survives navigation between the React and DOM-owned screens. There is no signup flow.

The conversation uses a persistent composer, warm user messages, compact assistant identity, recoverable message history and the existing live activity events. Suggestions fill the composer for review before sending. Progress is derived from actual runtime status; the visual refresh does not invent progress percentages, tasks or completed steps. The navigation can collapse on desktop and retains its preference. Opening an artifact hides desktop navigation as before; narrow screens use the existing full-screen preview. Administration screens share the same surfaces, form controls, headings and colors.

Motion is short and functional. Reduced-motion preferences disable animations, including pseudo-element activity indicators. Focus indicators remain visible, navigation retains accessible names when collapsed, and mobile layouts avoid horizontal page overflow. Built-in browser speech availability still controls the microphone and Live actions.

## Local artwork

`apps/web/src/assets/phoenix-wing.webp` was generated with the built-in image generator and converted to WebP without altering its composition. It is decorative (`alt=""`) and is bundled by Vite, then served by the Rust gateway. The production asset is approximately 109 KiB. The original PNG remains outside the repository in the generator output directory.

Generation prompt:

> Create a premium abstract brand artwork to use as the left half background of the LOGIN page of Fénix, a personal AI assistant. ARTWORK ONLY: no text, no letters, no logos, no UI, no mockup, no watermark. Portrait approximately 2:3. Dark almost-black green charcoal background #182526 that is completely quiet and uniform across the upper 40 percent, leaving space for white text which will be rendered by the website itself. In the LOWER two thirds, an elegant sculptural abstract phoenix wing made of layered curving ribbons of terracotta, copper and warm pale peach, inspired by finely folded handmade paper and satin brushed metal. Wide sweeping ribbons rise gracefully toward the middle-right of the composition from the bottom-left, with light, warm highlights and soft depth, sculptural folds and restrained fine tactile texture. The subject is an evocative abstract wing or flame, not a literal bird and not fire. Cinematic but soft side lighting, editorial sophistication, calm and welcoming. Keep upper portion absolutely uncluttered, subject mainly lower 65 percent. Rich detail in ribbons with dark shadow contrast. No neon, no purple, no gradients that look like generic AI branding. This should match a warm ivory + charcoal + terracotta productivity software design.

For reproducible builds while another checkout task is updating the frontend, build a stable frontend snapshot and set `FENIX_WEB_DIST` to its absolute `dist` path when compiling the Rust server. The default remains `apps/web/dist`.

## Browser validation

Set `FENIX_TEST_BINARY` to test a saved server executable without another build replacing it. Browser fixtures accept optional `FENIX_TEST_APP_PORT`, `FENIX_TEST_GATEWAY_PORT`, `FENIX_TEST_RUNTIME_PORT` and `FENIX_TEST_SEARCH_PORT` environment variables. Use distinct ports and Playwright's `--output` directory when another test run is active in the same checkout. `FENIX_TEST_EXPECT_TIMEOUT` can accommodate slower local machines without changing the production runtime.

[Español](../es/interface.md)


## Section URLs and loading

React Router manages `/chat`, `/projects`, `/memory`, `/skills`, `/browser`, `/schedules`, `/activity` and `/settings`. The former `/design` page redirects to `/chat`, and `/design?template=<id>` opens that template preview over the chat. Direct links, reloads and browser history retain the selected section; unauthenticated visitors sign in on the requested URL. The gateway and standalone server serve these explicit application routes without intercepting unknown API or asset paths.

The Browser screen at `/browser` provides owner login, persistent profiles, agent handoff, tab selection, viewport sizes and the encrypted password vault. Direct owner input uses browser APIs without entering the agent conversation. Shared loading states use a flame, indeterminate progress and skeleton lines, with accessible status text and reduced-motion support.

## Markdown rendering

Conversation history, progressive responses, skill instructions, review explanations, memory content, project briefs and plain document previews share a Markdown renderer. It supports headings, emphasis, lists, task lists, quotes, links, tables and fenced code blocks. Stored content and editors retain the original Markdown; JSON reports and before/after code diffs remain literal.

Marked parses the text and DOMPurify sanitizes the result. Raw HTML is displayed as text, unsafe link protocols are removed, and Markdown images do not trigger external requests. Code and tables scroll within their containers on narrow screens. This rendering runs in the browser and makes no additional model calls.
