/** What the page reports on its own: errors, and the downloads a browser would save. tour.ts imports this first. */

// ---- page errors, collected from the moment the tour is injected -----------------

export const pageErrors: string[] = [];
window.addEventListener("error", (event) => pageErrors.push(`error: ${event.message} @ ${event.filename}:${event.lineno}`));
window.addEventListener("unhandledrejection", (event) => pageErrors.push(`unhandled rejection: ${String((event.reason as Error)?.message ?? JSON.stringify(event.reason))}`));
const consoleError = console.error.bind(console);
console.error = (...args: unknown[]) => { pageErrors.push(`console.error: ${args.map((arg) => typeof arg === "string" ? arg : (arg as Error)?.message ?? JSON.stringify(arg)).join(" ")}`); consoleError(...args); };

// Downloads in a browser: the link is recorded and fetched by the tour instead of saved.
export const downloads: string[] = [];
document.addEventListener("click", (event) => {
  const link = (event.target as Element | null)?.closest?.("a[download]") as HTMLAnchorElement | null;
  if (!link) return;
  event.preventDefault();
  downloads.push(link.href);
}, true);
