import type { OscArg } from "../lib/api";
import { useT } from "../lib/i18n";

/** An argument as edited in the UI: the value stays a string until it is sent. */
export type ArgRow = { type: OscArg["type"]; value: string };

export function toOscArg(row: ArgRow): OscArg {
  switch (row.type) {
    case "int": return { type: "int", value: parseInt(row.value) || 0 };
    case "long": return { type: "long", value: parseInt(row.value) || 0 };
    case "float": return { type: "float", value: parseFloat(row.value) || 0 };
    case "double": return { type: "double", value: parseFloat(row.value) || 0 };
    case "bool": return { type: "bool", value: row.value === "true" };
    case "str": return { type: "str", value: row.value };
    case "blob": return { type: "blob", value: row.value ? row.value.split(" ").map(byte => parseInt(byte, 16)) : [] };
    default: return { type: "nil" };
  }
}

/** The inverse of `toOscArg`, for loading a stored signal into the editor. */
export function fromOscArg(a: OscArg): ArgRow {
  switch (a.type) {
    case "nil": return { type: "nil", value: "" };
    case "bool": return { type: "bool", value: a.value ? "true" : "false" };
    case "blob": return { type: "blob", value: a.value.map(byte => byte.toString(16).padStart(2, "0")).join(" ") };
    default: return { type: a.type, value: String(a.value) };
  }
}

export function fmtArg(a: OscArg): string {
  switch (a.type) {
    case "nil": return "nil";
    case "blob": return `blob[${a.value.length}]`;
    case "bool": return a.value ? "true" : "false";
    default: return String(a.value);
  }
}

const TYPES: OscArg["type"][] = ["int", "float", "str", "bool", "long", "double", "nil"];

/** Accepts only an argument list this editor could have produced (e.g. read back from storage). */
export function isArgRows(value: unknown): boolean {
  return Array.isArray(value) && value.every((row) =>
    typeof row === "object" && row !== null && typeof row.value === "string"
    && (TYPES.includes(row.type) || row.type === "blob"));
}

/** Typed OSC argument list editor, shared by the OSC and Broadcast senders. */
export function OscArgsEditor({
  args,
  onChange,
  disabled,
  onSubmit,
}: {
  args: ArgRow[];
  onChange: (next: ArgRow[]) => void;
  disabled?: boolean;
  /** Fired when Enter is pressed in a value field. */
  onSubmit?: () => void;
}) {
  const t = useT();
  const patch = (i: number, next: Partial<ArgRow>) =>
    onChange(args.map((x, j) => (j === i ? { ...x, ...next } : x)));
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && onSubmit) {
      e.preventDefault();
      onSubmit();
    }
  };

  return (
    <>
      {args.map((a, i) => (
        <div className="row tight" key={i} style={{ marginBottom: 6 }}>
          <select
            style={{ flex: "0 0 92px" }}
            aria-label={t("common.argType", { n: i + 1 })}
            value={a.type}
            disabled={disabled}
            onChange={(e) => patch(i, { type: e.target.value as OscArg["type"] })}
          >
            {a.type === "blob" && <option value="blob">blob</option>}
            {/* `ty`, not `t` — that name is the translate function here. */}
            {TYPES.map((ty) => <option key={ty} value={ty}>{ty}</option>)}
          </select>
          {a.type === "bool" ? (
            <select aria-label={t("common.argValue", { n: i + 1 })} value={a.value} disabled={disabled} onChange={(e) => patch(i, { value: e.target.value })}>
              <option value="true">true</option>
              <option value="false">false</option>
            </select>
          ) : a.type === "nil" ? (
            <input aria-label={t("common.argValue", { n: i + 1 })} value="—" disabled readOnly />
          ) : (
            <input
              aria-label={t("common.argValue", { n: i + 1 })}
              value={a.value}
              readOnly={a.type === "blob"}
              disabled={disabled}
              onKeyDown={onKeyDown}
              onChange={(e) => patch(i, { value: e.target.value })}
            />
          )}
          <button
            className="ghost sm"
            style={{ flex: "0 0 auto" }}
            disabled={disabled}
            aria-label={t("common.removeArg", { n: i + 1 })}
            data-tip={t("common.removeArg", { n: i + 1 })}
            onClick={() => onChange(args.filter((_, j) => j !== i))}
          >
            ✕
          </button>
        </div>
      ))}
      <button
        className="ghost sm"
        disabled={disabled}
        onClick={() => onChange([...args, { type: "float", value: "0" }])}
      >
        {t("common.addArgument")}
      </button>
    </>
  );
}
