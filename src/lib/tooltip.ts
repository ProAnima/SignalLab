/**
 * Where a tooltip goes. Pure, so it is tested without a browser; the layer
 * that shows tooltips is `components/TooltipLayer.tsx`.
 *
 * Explanations live in tooltips, not on the screen: an element carries its
 * help as `data-tip`, and one layer shows it on hover and on keyboard focus.
 */

export interface Box { left: number; top: number; width: number; height: number }
export interface Size { width: number; height: number }
export interface Placement { left: number; top: number; side: "above" | "below" }

/** Space between the element and its tooltip. */
export const TIP_GAP = 6;
/** A tooltip never touches the window edge. */
export const TIP_MARGIN = 8;

/**
 * Above the element, centred on it — so a label's help never covers its
 * field; below when it does not fit above and fits better below. Always kept
 * inside the window.
 */
export function placeTip(target: Box, tip: Size, view: Size): Placement {
  const above = target.top - TIP_GAP - TIP_MARGIN;
  const below = view.height - (target.top + target.height) - TIP_GAP - TIP_MARGIN;
  const side = tip.height <= above || above >= below ? "above" : "below";
  const top = side === "above" ? target.top - TIP_GAP - tip.height : target.top + target.height + TIP_GAP;
  const centred = target.left + target.width / 2 - tip.width / 2;
  return {
    left: Math.round(clamp(centred, TIP_MARGIN, view.width - TIP_MARGIN - tip.width)),
    top: Math.round(clamp(top, TIP_MARGIN, view.height - TIP_MARGIN - tip.height)),
    side,
  };
}

/** Text entry inside a labelled element: hovering it is typing, not asking. */
export const ENTRY = "input, textarea, select, [contenteditable]";

function clamp(value: number, min: number, max: number) {
  // A tooltip larger than the window starts at the margin.
  return max < min ? min : Math.min(Math.max(value, min), max);
}
