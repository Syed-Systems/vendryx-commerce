/**
 * PORT of The Matrix Hours `WatchParts` — the six SVG layers that make up the
 * hero timepiece. Geometry, colours and construction are the source's.
 *
 * No "use client": these are pure SVG with no hooks or handlers, so they render
 * on the server and ship no JavaScript. The source needed them client-side only
 * because GSAP drove the exploded view.
 */
const GOLD = "#c9a96e"
const GOLD_DIM = "rgba(201, 169, 110, 0.3)"
const GOLD_FAINT = "rgba(201, 169, 110, 0.12)"
const WHITE = "#f5f5f0"
const WHITE_DIM = "rgba(245, 245, 240, 0.15)"

// The source's guard against hydration mismatches: Node and the browser format
// long floats differently, so every computed coordinate is rounded first.
const round = (num: number) => Number(num.toFixed(3))

const svgProps = {
  width: "100%",
  height: "100%",
  style: { overflow: "visible" as const },
}

export function BezelRing() {
  const markers = Array.from({ length: 60 }, (_, i) => {
    const angle = i * 6 * (Math.PI / 180)
    const isHour = i % 5 === 0
    const outerR = 138
    const innerR = isHour ? 126 : 131
    return (
      <line
        key={i}
        x1={round(150 + Math.sin(angle) * innerR)}
        y1={round(150 - Math.cos(angle) * innerR)}
        x2={round(150 + Math.sin(angle) * outerR)}
        y2={round(150 - Math.cos(angle) * outerR)}
        stroke={isHour ? WHITE : GOLD_DIM}
        strokeWidth={isHour ? 2.5 : 0.8}
        strokeLinecap="round"
      />
    )
  })

  return (
    <svg viewBox="0 0 300 300" {...svgProps}>
      <circle
        cx="150"
        cy="150"
        r="145"
        fill="none"
        stroke={GOLD}
        strokeWidth="3"
      />
      <circle
        cx="150"
        cy="150"
        r="142"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.5"
      />
      {Array.from({ length: 120 }, (_, i) => {
        const a = i * 3 * (Math.PI / 180)
        return (
          <line
            key={`k${i}`}
            x1={round(150 + Math.sin(a) * 143)}
            y1={round(150 - Math.cos(a) * 143)}
            x2={round(150 + Math.sin(a) * 146)}
            y2={round(150 - Math.cos(a) * 146)}
            stroke={GOLD_FAINT}
            strokeWidth="0.6"
          />
        )
      })}
      {markers}
      {[12, 3, 6, 9].map((h) => {
        const a = h * 30 * (Math.PI / 180)
        return (
          <text
            key={h}
            x={round(150 + Math.sin(a) * 118)}
            y={round(150 - Math.cos(a) * 118)}
            fill={GOLD}
            fontSize="11"
            fontFamily="var(--font-space-mono), monospace"
            textAnchor="middle"
            dominantBaseline="central"
            fontWeight="700"
          >
            {h}
          </text>
        )
      })}
    </svg>
  )
}

export function DialFace() {
  return (
    <svg viewBox="0 0 300 300" {...svgProps}>
      <circle cx="150" cy="150" r="110" fill="rgba(20, 20, 20, 0.9)" />
      <circle
        cx="150"
        cy="150"
        r="110"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.8"
      />
      {Array.from({ length: 72 }, (_, i) => {
        const a = i * 5 * (Math.PI / 180)
        return (
          <line
            key={i}
            x1={round(150 + Math.sin(a) * 30)}
            y1={round(150 - Math.cos(a) * 30)}
            x2={round(150 + Math.sin(a) * 108)}
            y2={round(150 - Math.cos(a) * 108)}
            stroke={WHITE_DIM}
            strokeWidth="0.3"
          />
        )
      })}
      {Array.from({ length: 12 }, (_, i) => {
        const angle = i * 30 * (Math.PI / 180)
        const r = 95
        if (i === 0 || i === 3 || i === 6 || i === 9) return null
        return (
          <rect
            key={i}
            x={round(150 + Math.sin(angle) * r - 1.5)}
            y={round(150 - Math.cos(angle) * r - 6)}
            width="3"
            height="12"
            fill={GOLD}
            rx="0.5"
            transform={`rotate(${i * 30}, ${round(
              150 + Math.sin(angle) * r
            )}, ${round(150 - Math.cos(angle) * r)})`}
          />
        )
      })}
      <text
        x="150"
        y="105"
        fill={GOLD}
        fontSize="7"
        fontFamily="var(--font-space-mono), monospace"
        textAnchor="middle"
        letterSpacing="3"
        fontWeight="700"
        opacity="0.8"
      >
        MATRIX HOURS
      </text>
      <circle
        cx="150"
        cy="200"
        r="22"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.6"
      />
      {Array.from({ length: 12 }, (_, i) => {
        const a = i * 30 * (Math.PI / 180)
        return (
          <line
            key={`s${i}`}
            x1={round(150 + Math.sin(a) * 18)}
            y1={round(200 - Math.cos(a) * 18)}
            x2={round(150 + Math.sin(a) * 21)}
            y2={round(200 - Math.cos(a) * 21)}
            stroke={GOLD_DIM}
            strokeWidth="0.5"
          />
        )
      })}
      <rect
        x="198"
        y="144"
        width="18"
        height="12"
        rx="1.5"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.6"
      />
      <text
        x="207"
        y="153"
        fill={WHITE}
        fontSize="8"
        fontFamily="var(--font-space-mono), monospace"
        textAnchor="middle"
        fontWeight="700"
      >
        08
      </text>
    </svg>
  )
}

export function WatchHands() {
  return (
    <svg viewBox="0 0 300 300" {...svgProps}>
      <polygon points="148,155 150,70 152,155" fill={WHITE} opacity="0.95" />
      <line
        x1="150"
        y1="70"
        x2="150"
        y2="155"
        stroke={WHITE}
        strokeWidth="4.5"
        strokeLinecap="round"
        opacity="0.9"
      />
      <polygon points="149,158 150,42 151,158" fill={GOLD} opacity="0.9" />
      <line
        x1="150"
        y1="42"
        x2="150"
        y2="158"
        stroke={GOLD}
        strokeWidth="2.5"
        strokeLinecap="round"
        opacity="0.85"
      />
      <line
        x1="150"
        y1="35"
        x2="150"
        y2="170"
        stroke="#e53935"
        strokeWidth="0.8"
      />
      <circle cx="150" cy="170" r="3" fill="#e53935" opacity="0.7" />
      <circle cx="150" cy="150" r="5" fill={GOLD} />
      <circle cx="150" cy="150" r="3" fill="rgba(10,10,10,0.8)" />
      <circle cx="150" cy="150" r="1.5" fill={GOLD} />
    </svg>
  )
}

export function CaseBody() {
  return (
    <svg viewBox="0 0 300 340" {...svgProps}>
      <path
        d="M 95,40 Q 80,40 75,60 L 65,140 Q 60,170 65,200 L 75,280 Q 80,300 95,300 L 205,300 Q 220,300 225,280 L 235,200 Q 240,170 235,140 L 225,60 Q 220,40 205,40 Z"
        fill="none"
        stroke={GOLD}
        strokeWidth="1.5"
        opacity="0.7"
      />
      <path
        d="M 100,50 Q 88,50 84,68 L 76,140 Q 72,170 76,200 L 84,272 Q 88,290 100,290 L 200,290 Q 212,290 216,272 L 224,200 Q 228,170 224,140 L 216,68 Q 212,50 200,50 Z"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.6"
      />
      <rect
        x="237"
        y="158"
        width="18"
        height="24"
        rx="4"
        fill="none"
        stroke={GOLD}
        strokeWidth="1.2"
      />
      {[163, 167, 171, 175].map((y) => (
        <line
          key={y}
          x1="242"
          y1={y}
          x2="250"
          y2={y}
          stroke={GOLD_DIM}
          strokeWidth="0.5"
        />
      ))}
      <rect
        x="238"
        y="110"
        width="12"
        height="10"
        rx="2"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.8"
      />
      <rect
        x="238"
        y="220"
        width="12"
        height="10"
        rx="2"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.8"
      />
      {[
        [110, 42],
        [190, 42],
        [110, 298],
        [190, 298],
      ].map(([x, y], i) => (
        <circle
          key={i}
          cx={x}
          cy={y}
          r="3"
          fill="none"
          stroke={GOLD_DIM}
          strokeWidth="0.6"
        />
      ))}
    </svg>
  )
}

export function Movement() {
  const gearTeeth = (cx: number, cy: number, r: number, teeth: number) => {
    const pts: string[] = []
    for (let i = 0; i < teeth; i++) {
      const a1 = (i / teeth) * Math.PI * 2
      const a2 = ((i + 0.3) / teeth) * Math.PI * 2
      const a3 = ((i + 0.5) / teeth) * Math.PI * 2
      const a4 = ((i + 0.8) / teeth) * Math.PI * 2
      pts.push(
        `${round(cx + Math.cos(a1) * r)},${round(cy + Math.sin(a1) * r)}`
      )
      pts.push(
        `${round(cx + Math.cos(a2) * (r + 4))},${round(
          cy + Math.sin(a2) * (r + 4)
        )}`
      )
      pts.push(
        `${round(cx + Math.cos(a3) * (r + 4))},${round(
          cy + Math.sin(a3) * (r + 4)
        )}`
      )
      pts.push(
        `${round(cx + Math.cos(a4) * r)},${round(cy + Math.sin(a4) * r)}`
      )
    }
    return pts.join(" ")
  }

  return (
    <svg viewBox="0 0 300 300" {...svgProps}>
      <circle
        cx="150"
        cy="150"
        r="105"
        fill="rgba(15,15,15,0.6)"
        stroke={GOLD_DIM}
        strokeWidth="0.6"
      />
      <polygon
        points={gearTeeth(150, 150, 40, 24)}
        fill="none"
        stroke={GOLD}
        strokeWidth="0.8"
        opacity="0.6"
      />
      <circle
        cx="150"
        cy="150"
        r="38"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
      />
      <circle
        cx="150"
        cy="150"
        r="8"
        fill="none"
        stroke={GOLD}
        strokeWidth="1"
      />
      <polygon
        points={gearTeeth(190, 100, 18, 15)}
        fill="none"
        stroke={GOLD}
        strokeWidth="0.7"
        opacity="0.5"
      />
      <circle cx="190" cy="100" r="3" fill={GOLD} opacity="0.4" />
      <polygon
        points={gearTeeth(105, 110, 28, 20)}
        fill="none"
        stroke={GOLD}
        strokeWidth="0.6"
        opacity="0.4"
      />
      <circle
        cx="105"
        cy="110"
        r="26"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
      />
      <circle
        cx="105"
        cy="110"
        r="5"
        fill="none"
        stroke={GOLD}
        strokeWidth="0.8"
      />
      <polygon
        points={gearTeeth(170, 190, 22, 18)}
        fill="none"
        stroke={GOLD}
        strokeWidth="0.6"
        opacity="0.45"
      />
      <circle cx="170" cy="190" r="4" fill={GOLD} opacity="0.3" />
      <circle
        cx="200"
        cy="200"
        r="30"
        fill="none"
        stroke={GOLD}
        strokeWidth="1"
        opacity="0.5"
      />
      <circle
        cx="200"
        cy="200"
        r="28"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.3"
        strokeDasharray="2 3"
      />
      <circle cx="200" cy="200" r="4" fill={GOLD} opacity="0.5" />
      <line
        x1="200"
        y1="170"
        x2="200"
        y2="230"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
      />
      <line
        x1="170"
        y1="200"
        x2="230"
        y2="200"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
      />
      <path
        d="M 90,150 Q 120,135 150,150"
        fill="none"
        stroke={GOLD}
        strokeWidth="0.8"
        opacity="0.3"
      />
      <path
        d="M 150,150 Q 175,140 195,100"
        fill="none"
        stroke={GOLD}
        strokeWidth="0.8"
        opacity="0.3"
      />
      {[
        [150, 150],
        [190, 100],
        [105, 110],
        [170, 190],
        [200, 200],
      ].map(([x, y], i) => (
        <circle
          key={`j${i}`}
          cx={x}
          cy={y}
          r="2"
          fill="none"
          stroke="#e53935"
          strokeWidth="0.5"
          opacity="0.6"
        />
      ))}
      {[
        [120, 80],
        [180, 80],
        [100, 200],
        [130, 220],
      ].map(([x, y], i) => (
        <g key={`s${i}`}>
          <circle
            cx={x}
            cy={y}
            r="3.5"
            fill="none"
            stroke={GOLD_DIM}
            strokeWidth="0.5"
          />
          <line
            x1={x - 2}
            y1={y}
            x2={x + 2}
            y2={y}
            stroke={GOLD_DIM}
            strokeWidth="0.4"
          />
        </g>
      ))}
    </svg>
  )
}

export function Strap() {
  return (
    <svg viewBox="0 0 300 400" {...svgProps}>
      <path
        d="M 120,10 Q 115,10 112,20 L 108,160 Q 107,170 112,175 L 188,175 Q 193,170 192,160 L 188,20 Q 185,10 180,10 Z"
        fill="none"
        stroke={GOLD}
        strokeWidth="1.2"
        opacity="0.6"
      />
      <path
        d="M 118,20 L 115,165"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
        strokeDasharray="3 4"
      />
      <path
        d="M 182,20 L 185,165"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
        strokeDasharray="3 4"
      />
      <rect
        x="120"
        y="90"
        width="60"
        height="8"
        rx="2"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.5"
      />
      <rect
        x="122"
        y="120"
        width="56"
        height="8"
        rx="2"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.5"
      />
      <path
        d="M 120,225 Q 115,230 112,240 L 106,370 Q 104,385 115,390 L 185,390 Q 196,385 194,370 L 188,240 Q 185,230 180,225 Z"
        fill="none"
        stroke={GOLD}
        strokeWidth="1.2"
        opacity="0.6"
      />
      <path
        d="M 118,235 L 113,380"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
        strokeDasharray="3 4"
      />
      <path
        d="M 182,235 L 187,380"
        fill="none"
        stroke={GOLD_DIM}
        strokeWidth="0.4"
        strokeDasharray="3 4"
      />
      {[270, 290, 310, 330, 350].map((y) => (
        <ellipse
          key={y}
          cx="150"
          cy={y}
          rx="4"
          ry="2.5"
          fill="none"
          stroke={GOLD_DIM}
          strokeWidth="0.5"
        />
      ))}
      <rect
        x="115"
        y="225"
        width="70"
        height="15"
        rx="2"
        fill="none"
        stroke={GOLD}
        strokeWidth="1"
        opacity="0.7"
      />
      <line
        x1="150"
        y1="226"
        x2="150"
        y2="239"
        stroke={GOLD_DIM}
        strokeWidth="0.6"
      />
      <line
        x1="112"
        y1="178"
        x2="188"
        y2="178"
        stroke={GOLD}
        strokeWidth="1.5"
        strokeLinecap="round"
        opacity="0.4"
      />
      <line
        x1="112"
        y1="222"
        x2="188"
        y2="222"
        stroke={GOLD}
        strokeWidth="1.5"
        strokeLinecap="round"
        opacity="0.4"
      />
    </svg>
  )
}
