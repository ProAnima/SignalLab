/**
 * ProAnimaStudio mark: a signal pulse cut out of a gradient tile. Rendered as
 * inline SVG so it stays crisp at any DPI and needs no asset pipeline.
 */
export function BrandMark({ size = 30 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
      className="brand-mark"
    >
      <defs>
        <linearGradient id="pas-tile" x1="0" y1="0" x2="32" y2="32" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stopColor="#5df0bd" />
          <stop offset="55%" stopColor="#35c8e8" />
          <stop offset="100%" stopColor="#8b7cff" />
        </linearGradient>
      </defs>
      <rect x="0.5" y="0.5" width="31" height="31" rx="9.5" fill="url(#pas-tile)" />
      <rect
        x="0.5"
        y="0.5"
        width="31"
        height="31"
        rx="9.5"
        stroke="rgba(255,255,255,0.28)"
        strokeWidth="1"
      />
      <path
        d="M5 16.5h4.2l2-6.4 3.1 12.2 2.5-8.1 1.6 2.3H27"
        stroke="#08130f"
        strokeWidth="2.1"
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity="0.88"
      />
    </svg>
  );
}

/** Full lockup for the sidebar: mark + product name + studio byline. */
export function Brand({ name, by }: { name: string; by: string }) {
  return (
    <div className="brand">
      <BrandMark />
      <div className="brand-text">
        <span className="brand-name">{name}</span>
        <span className="brand-by">{by}</span>
      </div>
    </div>
  );
}
