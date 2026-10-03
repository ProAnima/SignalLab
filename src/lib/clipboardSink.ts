/**
 * The editor's Copy and Paste go through a hidden text field: the system's own
 * copy and paste of text, which every webview allows on a key press or a
 * click (a paste event on a button is not something WebKitGTK sends). The
 * field takes the focus for that moment only and gives it back.
 */
let field: HTMLTextAreaElement | null = null;

function sink(): HTMLTextAreaElement {
  if (field?.isConnected) return field;
  field = document.createElement("textarea");
  field.className = "clipboard-sink";
  field.tabIndex = -1;
  field.setAttribute("aria-hidden", "true");
  field.setAttribute("aria-label", "clipboard");
  document.body.append(field);
  return field;
}

/** The focus back where it was; where it was nowhere, off the field. */
function giveBack(element: HTMLTextAreaElement, back: Element | null) {
  if (back instanceof HTMLElement && back !== document.body && back !== element && back.isConnected) back.focus({ preventScroll: true });
  else element.blur();
}

/** On Ctrl+C or Ctrl+X, before the key's default: the key itself copies `text`. */
export function offer(text: string) {
  const element = sink();
  const back = document.activeElement;
  element.value = text;
  element.focus({ preventScroll: true });
  element.select();
  window.setTimeout(() => { element.value = ""; giveBack(element, back); }, 0);
}

/** On Ctrl+V, before the key's default: the text the key pastes goes to `take`. */
export function receive(take: (text: string) => void) {
  const element = sink();
  const back = document.activeElement;
  element.value = "";
  element.focus({ preventScroll: true });
  const finish = () => {
    element.removeEventListener("paste", pasted);
    window.clearTimeout(timer);
    element.value = "";
    giveBack(element, back);
  };
  const pasted = (event: ClipboardEvent) => {
    event.preventDefault();
    const text = event.clipboardData?.getData("text/plain") ?? "";
    finish();
    take(text);
  };
  element.addEventListener("paste", pasted);
  // Nothing pasted (an empty clipboard): the focus comes back all the same.
  const timer = window.setTimeout(finish, 1000);
}

/** From a button: the field, and the copy command a click allows. */
export function copyNow(text: string): boolean {
  const element = sink();
  const back = document.activeElement;
  element.value = text;
  element.focus({ preventScroll: true });
  element.select();
  let copied = false;
  try { copied = document.execCommand("copy"); } catch { copied = false; }
  element.value = "";
  giveBack(element, back);
  return copied;
}
