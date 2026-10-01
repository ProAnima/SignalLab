import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type { ExperimentNode } from "./api";
import { graphBounds, NODE_HEIGHT, NODE_WIDTH } from "./experimentGraph";

export function useExperimentViewport(nodes: ExperimentNode[] | undefined) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState({ width: 0, height: 0 });
  const [camera, setCamera] = useState({ zoom: 1, left: 0, top: 0 });
  const { zoom } = camera;
  const ready = nodes !== undefined;

  useEffect(() => {
    const element = scrollRef.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      // A hidden module has zero size. Retain its canvas extent and scroll position.
      if (element.clientWidth && element.clientHeight) setViewport({ width: element.clientWidth, height: element.clientHeight });
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [ready]);

  useLayoutEffect(() => {
    scrollRef.current?.scrollTo({ left: camera.left, top: camera.top });
  }, [camera]);

  const zoomAt = useCallback((value: number, anchor?: { x: number; y: number }) => {
    const element = scrollRef.current;
    if (!element) return;
    const next = Math.min(2, Math.max(.15, value));
    const x = anchor?.x ?? element.clientWidth / 2;
    const y = anchor?.y ?? element.clientHeight / 2;
    setCamera({ zoom: next, left: Math.max(0, (element.scrollLeft + x) * next / zoom - x),
      top: Math.max(0, (element.scrollTop + y) * next / zoom - y) });
  }, [zoom]);

  useEffect(() => {
    const element = scrollRef.current;
    if (!element) return;
    const wheel = (event: WheelEvent) => {
      if (!event.ctrlKey && !event.metaKey) return;
      event.preventDefault();
      const rect = element.getBoundingClientRect();
      const delta = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? element.clientHeight : 1);
      zoomAt(zoom * Math.exp(-delta * .002), { x: event.clientX - rect.left, y: event.clientY - rect.top });
    };
    element.addEventListener("wheel", wheel, { passive: false });
    return () => element.removeEventListener("wheel", wheel);
  }, [ready, zoom, zoomAt]);

  const fit = useCallback((items: ExperimentNode[]) => {
    const element = scrollRef.current;
    if (!element || !items.length) return;
    const bounds = graphBounds(items);
    const next = Math.max(.15, Math.min(1, (element.clientWidth - 64) / bounds.width, (element.clientHeight - 64) / bounds.height));
    setCamera({ zoom: next, left: Math.max(0, bounds.x * next - (element.clientWidth - bounds.width * next) / 2),
      top: Math.max(0, bounds.y * next - (element.clientHeight - bounds.height * next) / 2) });
  }, []);

  const reveal = useCallback((node: ExperimentNode) => {
    const element = scrollRef.current;
    if (!element) return;
    const next = Math.max(.8, zoom);
    setCamera({ zoom: next, left: Math.max(0, (node.x + NODE_WIDTH / 2) * next - element.clientWidth / 2),
      top: Math.max(0, (node.y + NODE_HEIGHT / 2) * next - element.clientHeight / 2) });
  }, [zoom]);

  /** Scroll just enough to show the node; unlike `reveal`, keeps zoom and a visible node still. */
  const ensureVisible = useCallback((node: ExperimentNode) => {
    const element = scrollRef.current;
    if (!element || !element.clientWidth) return;
    const margin = 24;
    const left = node.x * zoom - margin, right = (node.x + NODE_WIDTH) * zoom + margin;
    const top = node.y * zoom - margin, bottom = (node.y + NODE_HEIGHT) * zoom + margin;
    let x = element.scrollLeft, y = element.scrollTop;
    if (left < x) x = left; else if (right > x + element.clientWidth) x = right - element.clientWidth;
    if (top < y) y = top; else if (bottom > y + element.clientHeight) y = bottom - element.clientHeight;
    if (x !== element.scrollLeft || y !== element.scrollTop) setCamera({ zoom, left: Math.max(0, x), top: Math.max(0, y) });
  }, [zoom]);

  const bounds = graphBounds(nodes ?? []);
  // Space beyond the graph allows centering the last node without moving it.
  const canvasWidth = Math.max(viewport.width / zoom, bounds.x + bounds.width + viewport.width / (2 * zoom), (camera.left + viewport.width) / zoom);
  const canvasHeight = Math.max(viewport.height / zoom, bounds.y + bounds.height + viewport.height / (2 * zoom), (camera.top + viewport.height) / zoom);
  return { scrollRef, zoom, zoomAt, fit, reveal, ensureVisible, canvasWidth, canvasHeight };
}
