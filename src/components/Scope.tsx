import { useEffect, useRef } from "react";

interface Props {
  data: number[];
  min?: number;
  max?: number;
  color?: string;
  fill?: boolean;
  height?: number;
  grid?: boolean;
}

/** Rolling line plot on a canvas. Auto-ranges when min/max are omitted. */
export function Scope({
  data,
  min,
  max,
  color = "#38e0b0",
  fill = true,
  height = 160,
  grid = true,
}: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = height;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, w, h);

    // grid
    if (grid) {
      ctx.strokeStyle = "rgba(51,64,79,0.35)";
      ctx.lineWidth = 1;
      for (let i = 1; i < 4; i++) {
        const y = (h / 4) * i;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
      }
      for (let i = 1; i < 8; i++) {
        const x = (w / 8) * i;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
        ctx.stroke();
      }
    }

    if (data.length < 2) return;

    let lo = min ?? Math.min(...data);
    let hi = max ?? Math.max(...data);
    if (hi - lo < 1e-9) {
      hi = lo + 1;
      lo -= 1;
    }
    const pad = 8;
    const usable = h - pad * 2;
    const xstep = w / Math.max(1, data.length - 1);
    const yOf = (v: number) => pad + usable * (1 - (v - lo) / (hi - lo));

    ctx.beginPath();
    data.forEach((v, i) => {
      const x = i * xstep;
      const y = yOf(v);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });

    if (fill) {
      const grad = ctx.createLinearGradient(0, 0, 0, h);
      grad.addColorStop(0, color + "44");
      grad.addColorStop(1, color + "00");
      ctx.save();
      ctx.lineTo(w, h);
      ctx.lineTo(0, h);
      ctx.closePath();
      ctx.fillStyle = grad;
      ctx.fill();
      ctx.restore();
    }

    ctx.beginPath();
    data.forEach((v, i) => {
      const x = i * xstep;
      const y = yOf(v);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.6;
    ctx.lineJoin = "round";
    ctx.shadowColor = color;
    ctx.shadowBlur = 6;
    ctx.stroke();
  }, [data, min, max, color, fill, height, grid]);

  return <canvas ref={ref} style={{ height }} />;
}
