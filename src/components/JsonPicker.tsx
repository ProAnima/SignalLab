import type { ReactNode } from "react";

type Path = (string | number)[];

const MAX_NODES = 2000;

function count(value: unknown, limit = MAX_NODES): number {
  let total = 0;
  const stack = [value];
  while (stack.length && total <= limit) {
    const item = stack.pop();
    total++;
    if (item && typeof item === "object") stack.push(...Object.values(item as object));
  }
  return total;
}

/**
 * A JSON value laid out like pretty-printed JSON, where every value (and every
 * key, for the value under it) can be clicked: "Extract as variable" starts
 * from the thing the user is looking at. Very large bodies fall back to text.
 */
export function JsonPicker({ value, onPick, title }: { value: unknown; onPick: (path: Path, value: unknown) => void; title: string }) {
  if (count(value) > MAX_NODES) return <pre>{JSON.stringify(value, null, 2)}</pre>;
  const pad = (depth: number) => "  ".repeat(depth);
  const render = (item: unknown, path: Path, depth: number): ReactNode => {
    if (Array.isArray(item)) {
      if (!item.length) return "[]";
      return <>{"[\n"}{item.map((child, index) => <span key={index}>{pad(depth + 1)}{render(child, [...path, index], depth + 1)}{index < item.length - 1 ? "," : ""}{"\n"}</span>)}{pad(depth)}{"]"}</>;
    }
    if (item && typeof item === "object") {
      const entries = Object.entries(item);
      if (!entries.length) return "{}";
      return <>{"{\n"}{entries.map(([key, child], index) => <span key={key}>{pad(depth + 1)}
        <button type="button" className="json-key" title={title} onClick={() => onPick([...path, key], child)}>{JSON.stringify(key)}</button>{": "}
        {render(child, [...path, key], depth + 1)}{index < entries.length - 1 ? "," : ""}{"\n"}</span>)}{pad(depth)}{"}"}</>;
    }
    return <button type="button" className={`json-value ${item === null ? "null" : typeof item}`} title={title} onClick={() => onPick(path, item)}>{JSON.stringify(item)}</button>;
  };
  return <pre className="json-picker">{render(value, [], 0)}</pre>;
}
