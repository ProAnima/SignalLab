import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { flagOf } from "../lib/flags";
import { LANGS, useI18n, type Lang } from "../lib/i18n";

/** A language's name in another language (`Intl.DisplayNames`), for finding it by a letter; empty where the system has none. */
function nameIn(code: string, inLanguage: string): string {
  try {
    return new Intl.DisplayNames([inLanguage], { type: "language" }).of(code) ?? "";
  } catch {
    return "";
  }
}

function Flag({ code }: { code: string }) {
  return <img className="flag" src={flagOf(code)} alt="" width={20} height={15} draggable={false} />;
}

/**
 * The language of the interface: a button with the current one's flag and
 * letters, opening a list of every language by its flag and its own name — the
 * name a person who reads it looks for, whatever the page is in now. The arrows,
 * Home and End move in the list, a letter jumps to the next language it starts
 * (its own name, its letters, or its name here or in English: "g" finds Deutsch),
 * Enter or a click chooses; Escape or a click elsewhere closes it. The list is a
 * popover the button opens: above everything, clipped by nothing.
 */
export function LanguageMenu() {
  const { lang, setLang, t } = useI18n();
  const [open, setOpen] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const id = useId();
  const current = LANGS.find((item) => item.code === lang) ?? LANGS[0];

  useEffect(() => {
    const menu = list.current;
    const opener = button.current;
    if (!menu || !opener) return;
    // Under the button, lined up with its outer edge (the left one in a right-to-left page).
    const place = () => {
      const rect = opener.getBoundingClientRect();
      menu.style.top = `${rect.bottom + 6}px`;
      if (getComputedStyle(opener).direction === "rtl") {
        menu.style.left = `${Math.max(8, rect.left)}px`;
        menu.style.right = "auto";
      } else {
        menu.style.right = `${Math.max(8, window.innerWidth - rect.right)}px`;
        menu.style.left = "auto";
      }
    };
    const before = (event: Event) => { if ((event as ToggleEvent).newState === "open") place(); };
    const toggled = (event: Event) => {
      const opened = (event as ToggleEvent).newState === "open";
      setOpen(opened);
      if (opened) menu.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus();
    };
    menu.addEventListener("beforetoggle", before);
    menu.addEventListener("toggle", toggled);
    window.addEventListener("resize", place);
    return () => {
      menu.removeEventListener("beforetoggle", before);
      menu.removeEventListener("toggle", toggled);
      window.removeEventListener("resize", place);
    };
  }, []);

  const close = () => {
    try { list.current?.hidePopover(); } catch { /* not shown */ }
    button.current?.focus();
  };
  const choose = (code: Lang) => {
    setLang(code);
    close();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const items = [...(list.current?.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]') ?? [])];
    const at = items.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => { event.preventDefault(); items[(to + items.length) % items.length]?.focus(); };
    switch (event.key) {
      case "ArrowDown": return move(at + 1);
      case "ArrowUp": return move(at - 1);
      case "Home": return move(0);
      case "End": return move(items.length - 1);
      case "Escape": event.preventDefault(); event.stopPropagation(); return close();
      case "Tab": return close();
    }
    // A letter: the next language whose own name, letters or name here starts with it.
    if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey && event.key.trim()) {
      const typed = event.key.toLocaleLowerCase();
      const starts = (item: HTMLButtonElement) => (item.dataset.find ?? "").split("|").some((word) => word.startsWith(typed));
      const next = [...items.slice(at + 1), ...items.slice(0, at + 1)].find(starts);
      if (next) { event.preventDefault(); next.focus(); }
    }
  };

  return <div className="lang-menu">
    <button ref={button} className="lang-menu-button" popoverTarget={id} aria-haspopup="menu" aria-expanded={open}
      aria-label={`${t("app.language")}: ${current.label}`} data-tip={`${t("app.language")} · ${current.label}`}
      onKeyDown={(event) => {
        if ((event.key === "ArrowDown" || event.key === "ArrowUp") && !open) {
          event.preventDefault();
          try { list.current?.showPopover(); } catch { /* already shown */ }
        }
      }}>
      <Flag code={current.code} />
      <span className="lang-menu-short">{current.short}</span>
      <span className="lang-menu-caret" aria-hidden="true">▾</span>
    </button>
    <div ref={list} id={id} popover="auto" className="lang-menu-list" role="menu" aria-label={t("app.language")} onKeyDown={onKeyDown}>
      {LANGS.map((item) => {
        const find = [item.label, item.short, nameIn(item.code, lang), nameIn(item.code, "en")].map((word) => word.toLocaleLowerCase()).join("|");
        return <button key={item.code} role="menuitemradio" aria-checked={item.code === lang} tabIndex={-1} data-lang={item.code} data-find={find}
          onClick={() => choose(item.code)}>
          <Flag code={item.code} />
          <span className="lang-menu-name" lang={item.code} dir={item.dir}>{item.label}</span>
          <span className="lang-menu-check" aria-hidden="true">{item.code === lang ? "✓" : ""}</span>
        </button>;
      })}
    </div>
  </div>;
}
