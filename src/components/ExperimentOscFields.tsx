import { useEffect, useRef, useState } from "react";
import type { OscArg } from "../lib/api";
import { OscArgsEditor, fromOscArg, toOscArg, type ArgRow } from "./OscArgs";

/** Keep intermediate numeric input such as "-" or "0." while typing. */
export function ExperimentOscFields({ args, onChange }: { args: OscArg[]; onChange: (args: OscArg[]) => void }) {
  const [rows, setRows] = useState(() => args.map(fromOscArg));
  const lastEmitted = useRef(args);
  useEffect(() => {
    if (args !== lastEmitted.current) setRows(args.map(fromOscArg));
    lastEmitted.current = args;
  }, [args]);
  const change = (next: ArgRow[]) => {
    setRows(next);
    const converted = next.map(toOscArg);
    lastEmitted.current = converted;
    onChange(converted);
  };
  return <OscArgsEditor args={rows} onChange={change} />;
}
