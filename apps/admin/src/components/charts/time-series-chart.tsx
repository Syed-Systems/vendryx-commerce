import { clx } from "@medusajs/ui"
import { useMemo, useRef, useState } from "react"

/**
 * Single-series area chart for the reporting endpoints.
 *
 * Hand-rolled SVG rather than a charting library: the admin bundle already ships a
 * 4.4 MB entry chunk and trips Vite's size warning, and one area chart plus a
 * crosshair is far less code than the smallest library that would draw it. It also
 * means the marks inherit the theme through `currentColor` instead of needing the
 * theme read in JS and passed down.
 *
 * Colour: one hue, `--ui-fg-interactive`'s ramp. Both steps were checked against the
 * light (#FFFFFF) and dark (#212124) chart surfaces for the OKLCH lightness band,
 * chroma floor, and 3:1 contrast. The dark band sits *lower* than the light one, so
 * dark mode uses a slightly deeper step than the interactive token rather than a
 * lighter one. Each chart plots a single series, so there is no categorical pair to
 * separate and no legend box — the heading names what is plotted.
 */

export interface TimeSeriesPoint {
  /** ISO date, `YYYY-MM-DD`. */
  date: string
  value: number
}

interface TimeSeriesChartProps {
  data: TimeSeriesPoint[]
  /** Renders the value in the tooltip and the endpoint label. */
  format: (value: number) => string
  /** Describes the series for screen readers, e.g. "Revenue per day". */
  label: string
  className?: string
  height?: number
}

const PAD = { top: 16, right: 16, bottom: 26, left: 52 }
const GRID_LINES = 4

/** Rounds an axis maximum up to a clean 1/2/5 × 10ⁿ step. */
function niceMax(value: number): number {
  if (value <= 0) {
    return 1
  }
  const magnitude = 10 ** Math.floor(Math.log10(value))
  const normalized = value / magnitude
  const step = normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10
  return step * magnitude
}

function formatTick(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toLocaleString(undefined, { maximumFractionDigits: 1 })}M`
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toLocaleString(undefined, { maximumFractionDigits: 1 })}k`
  }
  return value.toLocaleString()
}

function formatDate(iso: string): string {
  const parsed = new Date(`${iso}T00:00:00`)
  if (Number.isNaN(parsed.getTime())) {
    return iso
  }
  return parsed.toLocaleDateString(undefined, { month: "short", day: "numeric" })
}

export const TimeSeriesChart = ({
  data,
  format,
  label,
  className,
  height = 220,
}: TimeSeriesChartProps) => {
  const svgRef = useRef<SVGSVGElement>(null)
  const [activeIndex, setActiveIndex] = useState<number | null>(null)
  const [width, setWidth] = useState(720)

  const geometry = useMemo(() => {
    const max = niceMax(Math.max(...data.map((d) => d.value), 0))
    const plotWidth = Math.max(width - PAD.left - PAD.right, 1)
    const plotHeight = Math.max(height - PAD.top - PAD.bottom, 1)

    const x = (index: number) =>
      data.length <= 1
        ? PAD.left + plotWidth / 2
        : PAD.left + (index / (data.length - 1)) * plotWidth
    const y = (value: number) => PAD.top + plotHeight - (value / max) * plotHeight

    const points = data.map((point, index) => ({
      ...point,
      cx: x(index),
      cy: y(point.value),
    }))

    const line = points.map((p) => `${p.cx},${p.cy}`).join(" ")
    const baseline = PAD.top + plotHeight
    const area = points.length
      ? `M ${points[0].cx},${baseline} L ${line.split(" ").join(" L ")} L ${
          points[points.length - 1].cx
        },${baseline} Z`
      : ""

    return { max, points, line, area, baseline, plotHeight }
  }, [data, width, height])

  // The crosshair snaps to the nearest x, so the reader aims at a date rather than
  // at a 2px line.
  const handlePointerMove = (event: React.PointerEvent<SVGSVGElement>) => {
    const svg = svgRef.current
    if (!svg || !data.length) {
      return
    }
    const bounds = svg.getBoundingClientRect()
    const ratio = (event.clientX - bounds.left) / bounds.width
    const scaled = ratio * width
    let nearest = 0
    let smallest = Infinity
    geometry.points.forEach((point, index) => {
      const distance = Math.abs(point.cx - scaled)
      if (distance < smallest) {
        smallest = distance
        nearest = index
      }
    })
    setActiveIndex(nearest)
  }

  if (!data.length) {
    return (
      <div
        className={clx(
          "text-ui-fg-muted flex items-center justify-center text-sm",
          className
        )}
        style={{ height }}
      >
        No data recorded in this window.
      </div>
    )
  }

  const active = activeIndex === null ? null : geometry.points[activeIndex]
  const last = geometry.points[geometry.points.length - 1]

  return (
    <div
      className={clx("relative w-full text-[#3B82F6] dark:text-[#5591F5]", className)}
      ref={(node) => {
        if (node && node.clientWidth && node.clientWidth !== width) {
          setWidth(node.clientWidth)
        }
      }}
    >
      <svg
        ref={svgRef}
        width="100%"
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        role="img"
        aria-label={label}
        className="overflow-visible"
        onPointerMove={handlePointerMove}
        onPointerLeave={() => setActiveIndex(null)}
      >
        {/* Gridlines: hairline, solid, one step off the surface. */}
        {Array.from({ length: GRID_LINES + 1 }).map((_, index) => {
          const value = (geometry.max / GRID_LINES) * index
          const y = geometry.baseline - (index / GRID_LINES) * geometry.plotHeight
          return (
            <g key={index}>
              <line
                x1={PAD.left}
                x2={width - PAD.right}
                y1={y}
                y2={y}
                className="stroke-ui-border-base"
                strokeWidth={1}
              />
              <text
                x={PAD.left - 10}
                y={y + 4}
                textAnchor="end"
                className="fill-ui-fg-muted text-xs tabular-nums"
              >
                {formatTick(value)}
              </text>
            </g>
          )
        })}

        {/* Area wash at ~10% — never a saturated block. */}
        <path d={geometry.area} fill="currentColor" fillOpacity={0.1} />
        <polyline
          points={geometry.line}
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        />

        {/* Endpoint marker, ringed in the surface colour so it stays legible. */}
        <circle
          cx={last.cx}
          cy={last.cy}
          r={4}
          fill="currentColor"
          className="stroke-ui-bg-base"
          strokeWidth={2}
        />

        {active ? (
          <g>
            <line
              x1={active.cx}
              x2={active.cx}
              y1={PAD.top}
              y2={geometry.baseline}
              className="stroke-ui-fg-muted"
              strokeWidth={1}
            />
            <circle
              cx={active.cx}
              cy={active.cy}
              r={4}
              fill="currentColor"
              className="stroke-ui-bg-base"
              strokeWidth={2}
            />
          </g>
        ) : null}

        {/* X labels: first and last only — a date under every point is unreadable. */}
        <text
          x={PAD.left}
          y={height - 8}
          textAnchor="start"
          className="fill-ui-fg-muted text-xs"
        >
          {formatDate(data[0].date)}
        </text>
        <text
          x={width - PAD.right}
          y={height - 8}
          textAnchor="end"
          className="fill-ui-fg-muted text-xs"
        >
          {formatDate(data[data.length - 1].date)}
        </text>
      </svg>

      {/* Value leads, label follows — the reader already knows the series. */}
      {active ? (
        <div
          className="bg-ui-bg-base shadow-elevation-tooltip pointer-events-none absolute top-2 z-10 rounded-lg px-3 py-2"
          style={{
            left: `${Math.min(Math.max((active.cx / width) * 100, 6), 78)}%`,
          }}
        >
          <div className="text-ui-fg-base text-sm font-medium tabular-nums">
            {format(active.value)}
          </div>
          <div className="text-ui-fg-subtle text-xs">{formatDate(active.date)}</div>
        </div>
      ) : null}
    </div>
  )
}
