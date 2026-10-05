/** A request's authentication and body as the editor and the HTTP screen change them (pure). */
import type { HttpAuth } from "./api";

export const AUTH_SCHEMES: HttpAuth["scheme"][] = ["none", "basic", "bearer", "digest"];

/** A scheme chosen in the select: its credentials kept where they carry over (Basic ⇄ Digest). */
export function withScheme(auth: HttpAuth, scheme: HttpAuth["scheme"]): HttpAuth {
  const named = auth.scheme === "basic" || auth.scheme === "digest" ? { username: auth.username, password: auth.password } : { username: "", password: "" };
  switch (scheme) {
    case "none": return { scheme };
    case "bearer": return { scheme, token: auth.scheme === "bearer" ? auth.token : "" };
    default: return { scheme, ...named };
  }
}

/** Methods the HTTP screen offers a body for; GET and HEAD hide the field. */
export const takesBody = (method: string): boolean => method !== "GET" && method !== "HEAD";

/**
 * The body the HTTP screen sends, saves and adds to an experiment: none when
 * the field is hidden (GET, HEAD) or empty — a body typed earlier never rides
 * along unseen. The engine itself takes a body with any method.
 */
export const screenBody = (method: string, body: string): string | null => (takesBody(method) && body.length ? body : null);
