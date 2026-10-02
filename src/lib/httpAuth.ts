/** The authentication of a request as the editor and the HTTP screen change it (pure). */
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
