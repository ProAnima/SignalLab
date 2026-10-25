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

/** A side of an element a larger card can go on: `start`/`end` follow the reading direction. */
export type Side = "above" | "below" | "start" | "end";
/** The same side on the screen, as coordinates have it. */
export type ScreenSide = "above" | "below" | "left" | "right";
export interface CardPlacement { left: number; top: number; side: ScreenSide; arrow: number }

/** Space between an element and a card beside it (the card's arrow sits in it). */
export const CARD_GAP = 14;
/** The arrow keeps this far from the card's corners. */
export const ARROW_INSET = 18;

/**
 * Where a card larger than a tooltip goes beside `target` (the introduction
 * tour's): on the first side of `prefer` it fits on, else on the next of below,
 * above, end, start that does, else on the side with the most room — always
 * inside the window. `arrow` is where the card's edge facing the target points
 * at its centre: from the card's left (above, below) or top (left, right).
 */
export function placeCard(target: Box, card: Size, view: Size, prefer: readonly Side[] = [], rtl = false): CardPlacement {
  const onScreen = (side: Side): ScreenSide => side === "start" ? (rtl ? "right" : "left") : side === "end" ? (rtl ? "left" : "right") : side;
  const room: Record<ScreenSide, number> = {
    above: target.top - CARD_GAP - TIP_MARGIN,
    below: view.height - (target.top + target.height) - CARD_GAP - TIP_MARGIN,
    left: target.left - CARD_GAP - TIP_MARGIN,
    right: view.width - (target.left + target.width) - CARD_GAP - TIP_MARGIN,
  };
  const spare = (side: ScreenSide) => room[side] - (side === "above" || side === "below" ? card.height : card.width);
  const order = [...prefer, ...(["below", "above", "end", "start"] as const).filter((side) => !prefer.includes(side))].map(onScreen);
  const side = order.find((candidate) => spare(candidate) >= 0) ?? order.reduce((best, candidate) => spare(candidate) > spare(best) ? candidate : best);
  const centreX = target.left + target.width / 2;
  const centreY = target.top + target.height / 2;
  const across = side === "above" || side === "below";
  const left = across ? centreX - card.width / 2 : side === "left" ? target.left - CARD_GAP - card.width : target.left + target.width + CARD_GAP;
  const top = !across ? centreY - card.height / 2 : side === "above" ? target.top - CARD_GAP - card.height : target.top + target.height + CARD_GAP;
  const placed = {
    left: Math.round(clamp(left, TIP_MARGIN, view.width - TIP_MARGIN - card.width)),
    top: Math.round(clamp(top, TIP_MARGIN, view.height - TIP_MARGIN - card.height)),
  };
  const arrow = across
    ? clamp(centreX - placed.left, ARROW_INSET, card.width - ARROW_INSET)
    : clamp(centreY - placed.top, ARROW_INSET, card.height - ARROW_INSET);
  return { ...placed, side, arrow: Math.round(arrow) };
}

/** Text entry inside a labelled element: hovering it is typing, not asking. */
export const ENTRY = "input, textarea, select, [contenteditable]";

function clamp(value: number, min: number, max: number) {
  // A tooltip larger than the window starts at the margin.
  return max < min ? min : Math.min(Math.max(value, min), max);
}
