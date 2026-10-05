import type { MarkdownRenderer } from "vitepress";

/** `[[version]]`: the release the documentation is built with (package.json). */
export const VERSION_MARK = "[[version]]";

/**
 * Examples name the current release — an image tag, a download, the Action's
 * ref — as `[[version]]`, in text and in code blocks, so they never fall a
 * release behind. A test refuses the number written out.
 */
export function versionMarks(md: MarkdownRenderer, version: string) {
  md.core.ruler.after("block", "signal-lab-version", (state) => {
    for (const token of state.tokens) {
      if (token.content.includes(VERSION_MARK)) token.content = token.content.replaceAll(VERSION_MARK, version);
    }
  });
}
