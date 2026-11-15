// Portraits are interface assets looked up by character id (avatar R10), with a sleeping one per character
// (avatar FR-016).

const urls = import.meta.glob<string>("./assets/characters/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});

const byId: Record<string, string> = Object.fromEntries(
  Object.entries(urls).map(([path, url]) => [path.slice(path.lastIndexOf("/") + 1, -".svg".length), url]),
);
const first = Object.keys(byId).find((id) => !id.endsWith("-sleeping")) ?? "";

/** An unknown id, for example from a hand-edited IDENTITY.md, shows the first character's portrait. */
export function portrait(id: string, options: { sleeping?: boolean } = {}): string {
  const known = id in byId ? id : first;
  return (options.sleeping && byId[`${known}-sleeping`]) || byId[known];
}
