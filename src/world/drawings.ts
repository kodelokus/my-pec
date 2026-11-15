// The world's drawings: bundled pixel-art SVGs looked up by item id (world R9, journey R4). Their inner markup becomes a `<symbol>`,
// so the companion's palette, set as CSS custom properties, colours every item.

const sources = import.meta.glob<string>("../assets/world/*.svg", {
  eager: true,
  query: "?raw",
  import: "default",
});

const byId: Record<string, string> = Object.fromEntries(
  Object.entries(sources).map(([path, svg]) => [
    path.slice(path.lastIndexOf("/") + 1, -".svg".length),
    svg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, ""),
  ]),
);

/** Shown for an id this version doesn't know (world FR-015). */
const PLACEHOLDER = '<rect x="9" y="9" width="14" height="14" rx="4" fill="var(--w-stone)" opacity="0.6"/>';

/** The inner SVG markup of an item, or the neutral placeholder. Bundled assets only, never model output. */
export const drawing = (id: string): string => byId[id] ?? PLACEHOLDER;

export const hasDrawing = (id: string): boolean => id in byId;
