import type { EngineError } from "./api";
import { en } from "./locales/en.ts";
import type { Translate } from "./i18n";

/**
 * One way to show any failure. The engine rejects with an `EngineError` (a
 * code, values, the node and field it is about, the system's own wording);
 * our own code can throw an `Error`, and a dictionary key or a stored older
 * text passes through as it is. Everything that shows a failure — banners, the timeline, the
 * console, Send now, panels — goes through `describeError`, so a failure reads
 * the same everywhere and re-renders when the language changes: callers keep
 * the failure itself, never the finished sentence.
 */

/** Anything a command, an event or our own code can fail with. */
export type Failure = unknown;

export function isEngineError(value: unknown): value is EngineError {
  return typeof value === "object" && value !== null && typeof (value as { code?: unknown }).code === "string";
}

/** A failure as a person reads it. */
export interface Described {
  /** What went wrong and why, in the current language. */
  message: string;
  /** Where: node, field and position, e.g. "HTTP request · Header value 2". Empty when nowhere in particular. */
  where: string;
  /** The underlying system, parser or library text, verbatim. */
  detail?: string;
  /** The node it is about, for "show me". */
  nodeId?: string;
  /** `where — message` for one-line places: the console, tooltips. */
  text: string;
}

const KNOWN = en as Record<string, string>;
const has = (key: string) => key in KNOWN;

/** The label of a field: `field.<key>`, numbered for repeated fields. */
export function fieldLabel(field: { key: string; index?: number }, t: Translate): string {
  const key = `field.${field.key}`;
  const name = has(key) ? t(key) : field.key;
  return field.index ? `${name} ${field.index}` : name;
}

/**
 * Values for an engine message (an error or a step result), with comparison
 * operators (`op`) worded in the current language.
 */
export function messageParams(params: Record<string, string | number> | undefined, t: Translate): Record<string, string | number> {
  const out: Record<string, string | number> = { ...params };
  if (typeof out.op === "string" && has(`exp.op.${out.op}`)) out.op = t(`exp.op.${out.op}`);
  return out;
}

/** Legacy engine text passes through `t` unchanged; a dictionary key is translated. */
function plainText(failure: Failure, t: Translate): string {
  if (failure instanceof Error) return failure.message;
  if (typeof failure === "string") return t(failure);
  try { return JSON.stringify(failure) ?? String(failure); } catch { return String(failure); }
}

/**
 * `nodeLabel` names a node for the reader (its type, say); without it, or for
 * a node that no longer exists, the location starts at the field.
 */
export function describeError(failure: Failure, t: Translate, nodeLabel?: (id: string) => string | null | undefined): Described {
  if (!isEngineError(failure)) {
    const message = plainText(failure, t);
    return { message, where: "", text: message };
  }
  const key = `err.${failure.code}`;
  const message = has(key) ? t(key, messageParams(failure.params, t)) : t("err.unknown", { code: failure.code });
  const where: string[] = [];
  const node = failure.node ? nodeLabel?.(failure.node) : null;
  if (node) where.push(node);
  // An emulator's problems say which rule (route) or retained message, and which response, they are in.
  for (const part of ["rule", "retained", "response"] as const) {
    const n = failure.params?.[part];
    if (n !== undefined && n !== "") where.push(t(`emu.where.${part}`, { n }));
  }
  if (failure.field) where.push(fieldLabel(failure.field, t));
  if (failure.params?.position) where.push(t("err.position", { position: failure.params.position }));
  const located = where.join(" · ");
  return {
    message,
    where: located,
    detail: failure.detail || undefined,
    nodeId: failure.node,
    text: located ? `${located} — ${message}` : message,
  };
}

/**
 * A response that never arrived, as a failure: with a known cause it reads like
 * the engine's `transport.<cause>` error about `target`, the full text as its
 * detail; otherwise the text as it is.
 */
export function responseFailure(response: { error: string | null; cause?: string | null }, target: string): Failure {
  if (!response.error) return null;
  if (!response.cause) return response.error;
  return { code: `transport.${response.cause}`, params: { target }, detail: response.error } satisfies EngineError;
}

/** The node a failure is about, if any. */
export const failureNode = (failure: Failure): string | undefined => isEngineError(failure) ? failure.node : undefined;
