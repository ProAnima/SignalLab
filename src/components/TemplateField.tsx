import { createContext, useContext, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { hasTemplate, templateAt } from "../lib/experimentData";
import { useT } from "../lib/i18n";

export interface TemplateSuggestion {
  /** Text placed between `{{` and `}}`. */
  insert: string;
  detail: string;
  group: "params" | "vars" | "secrets" | "generators";
}

/** What `{{` offers in the fields of the selected node; provided by the editor. */
export const TemplateSuggestions = createContext<TemplateSuggestion[]>([]);

const GROUP_TITLE = { params: "exp.suggest.params", vars: "exp.suggest.vars", secrets: "exp.suggest.secrets", generators: "exp.suggest.generators" } as const;

/**
 * A text field that knows the template language: typing `{{` (or Ctrl+Space)
 * lists parameters, upstream variables and generators; ↑/↓ choose, Enter or
 * Tab insert, Esc closes without leaving the field.
 */
export function TemplateField({ value, onChange, multiline, primary, placeholder, label }: {
  value: string;
  onChange: (value: string) => void;
  multiline?: boolean;
  primary?: boolean;
  placeholder?: string;
  label?: string;
}) {
  const t = useT();
  const suggestions = useContext(TemplateSuggestions);
  const field = useRef<HTMLInputElement & HTMLTextAreaElement>(null);
  const caretAfter = useRef<number | null>(null);
  const [open, setOpen] = useState<{ start: number; query: string } | null>(null);
  const [cursor, setCursor] = useState(0);

  const query = open?.query.toLowerCase() ?? "";
  const matches = open ? suggestions.filter((item) => item.insert.toLowerCase().startsWith(query)
    || (query.length > 1 && item.detail.toLowerCase().includes(query))).slice(0, 10) : [];
  const at = Math.min(cursor, Math.max(0, matches.length - 1));

  useLayoutEffect(() => {
    if (caretAfter.current === null || !field.current) return;
    field.current.setSelectionRange(caretAfter.current, caretAfter.current);
    caretAfter.current = null;
  }, [value]);

  const update = (text: string, caret: number) => {
    onChange(text);
    setOpen(templateAt(text, caret));
    setCursor(0);
  };

  const insert = (item: TemplateSuggestion) => {
    const element = field.current;
    const caret = element?.selectionStart ?? value.length;
    const place = templateAt(value, caret);
    if (!place) return;
    const after = value.slice(caret);
    const closing = after.startsWith("}}") ? "" : "}}";
    caretAfter.current = place.start + 2 + item.insert.length + 2;
    onChange(value.slice(0, place.start) + "{{" + item.insert + closing + after);
    setOpen(null);
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === " " && event.ctrlKey) {
      event.preventDefault();
      const caret = field.current?.selectionStart ?? value.length;
      caretAfter.current = caret + 2;
      update(value.slice(0, caret) + "{{" + value.slice(caret), caret + 2);
      return;
    }
    if (!open || !matches.length) return;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      setCursor((at + (event.key === "ArrowDown" ? 1 : -1) + matches.length) % matches.length);
    } else if ((event.key === "Enter" && !event.ctrlKey && !event.metaKey) || event.key === "Tab") {
      event.preventDefault(); event.stopPropagation();
      insert(matches[at]);
    } else if (event.key === "Escape") {
      // Close the list only; the form keeps focus.
      event.preventDefault(); event.stopPropagation();
      setOpen(null);
    }
  };

  const common = {
    ref: field,
    value,
    placeholder,
    "aria-label": label,
    "data-primary": primary || undefined,
    className: hasTemplate(value) ? "has-template" : undefined,
    "aria-expanded": open ? matches.length > 0 : undefined,
    onChange: (event: { target: HTMLInputElement | HTMLTextAreaElement }) => update(event.target.value, event.target.selectionStart ?? event.target.value.length),
    onKeyDown,
    onBlur: () => setOpen(null),
  };

  return <span className="template-field">
    {multiline ? <textarea {...common} /> : <input {...common} />}
    {open && matches.length > 0 && <span className="template-suggestions" role="listbox" aria-label={t("exp.suggest.vars")}>
      {matches.map((item, index) => <span key={item.group + item.insert} className="template-suggestion-row">
        {(index === 0 || matches[index - 1].group !== item.group) && <span className="template-suggestion-group">{t(GROUP_TITLE[item.group])}</span>}
        <button type="button" role="option" aria-selected={index === at} tabIndex={-1} className={index === at ? "active" : ""}
          onMouseDown={(event) => { event.preventDefault(); insert(item); }} onMouseEnter={() => setCursor(index)}>
          <code>{item.insert}</code><small>{item.detail}</small>
        </button>
      </span>)}
    </span>}
  </span>;
}
