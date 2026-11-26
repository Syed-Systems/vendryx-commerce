// Interactive 3D Rotating Architectural Wireframe & Double-Wall Cross-Section of KOVA Flask
export function initSpecsCanvas(
  canvas: HTMLCanvasElement | null,
  getMode: () => "light" | "dark"
): () => void {
  if (!canvas) return () => {}
  const ctx = canvas.getContext("2d")
  if (!ctx) return () => {}

  const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches
  let angle = 0.35
  const baseSpeed = 0.0085
  let hoverBoost = 0
  let rafId = 0
  let isVisible = true
  let disposed = false

  const resize = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2)
    const rect = canvas.getBoundingClientRect()
    const w = Math.max(200, Math.floor((rect.width || canvas.clientWidth || 440) * dpr))
    const h = Math.max(200, Math.floor((rect.height || canvas.clientHeight || 300) * dpr))
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w
      canvas.height = h
    }
  }

  const onEnter = () => {
    hoverBoost = 0.018
  }
  const onLeave = () => {
    hoverBoost = 0
  }

  const parent = canvas.parentElement
  parent?.addEventListener("pointerenter", onEnter)
  parent?.addEventListener("pointerleave", onLeave)

  // Outer shell profile rings
  const outerRings = [
    { y: -0.92, r: 0.44 },
    { y: -0.82, r: 0.48 },
    { y: 0.25, r: 0.48 },
    { y: 0.44, r: 0.36 },
    { y: 0.56, r: 0.34 },
    { y: 0.62, r: 0.37 },
    { y: 0.74, r: 0.37 },
    { y: 0.86, r: 0.24 },
    { y: 0.98, r: 0.24 },
  ]

  // Inner copper-lined vacuum chamber rings
  const innerRings = [
    { y: -0.78, r: 0.39 },
    { y: 0.22, r: 0.39 },
    { y: 0.42, r: 0.3 },
  ]

  const renderOnce = (advance: boolean) => {
    resize()
    const w = canvas.width
    const h = canvas.height
    ctx.clearRect(0, 0, w, h)

    const isDark = getMode() === "dark"
    const strokeMain = isDark
      ? "rgba(239, 243, 241, 0.38)"
      : "rgba(17, 22, 20, 0.34)"
    const strokeInner = isDark
      ? "rgba(116, 178, 128, 0.48)"
      : "rgba(61, 87, 64, 0.42)"
    const strokeAccent = isDark
      ? "rgba(116, 178, 128, 0.95)"
      : "rgba(61, 87, 64, 0.95)"

    const cx = w * 0.72
    const cy = h * 0.47
    const scaleY = Math.min(w, h) * 0.4
    const scaleX = scaleY * 0.82

    if (advance && !prefersReduced) {
      angle += baseSpeed + hoverBoost
    }

    ctx.lineWidth = Math.max(1.3, w / 420)

    // Draw outer architectural latitude rings
    for (let i = 0; i < outerRings.length; i++) {
      const { y, r } = outerRings[i]
      ctx.beginPath()
      ctx.strokeStyle = i === 4 || i === 5 ? strokeAccent : strokeMain
      const rx = r * scaleX
      const ry = r * scaleX * 0.22
      const py = cy - y * scaleY
      ctx.ellipse(cx, py, rx, ry, 0, 0, Math.PI * 2)
      ctx.stroke()
    }

    // Draw dashed inner copper vacuum chamber cross-section rings
    ctx.save()
    ctx.setLineDash([4, 4])
    for (let i = 0; i < innerRings.length; i++) {
      const { y, r } = innerRings[i]
      ctx.beginPath()
      ctx.strokeStyle = strokeInner
      const rx = r * scaleX
      const ry = r * scaleX * 0.22
      const py = cy - y * scaleY
      ctx.ellipse(cx, py, rx, ry, 0, 0, Math.PI * 2)
      ctx.stroke()
    }
    ctx.restore()

    // Rotating 3D vertical meridians
    const meridians = 12
    for (let m = 0; m < meridians; m++) {
      const theta = angle + (m * Math.PI * 2) / meridians
      const cosT = Math.cos(theta)
      const sinT = Math.sin(theta)
      ctx.beginPath()
      ctx.strokeStyle = m % 3 === 0 ? strokeAccent : strokeMain
      for (let i = 0; i < outerRings.length; i++) {
        const { y, r } = outerRings[i]
        const px = cx + cosT * r * scaleX
        const py = cy - y * scaleY + sinT * r * scaleX * 0.22
        if (i === 0) ctx.moveTo(px, py)
        else ctx.lineTo(px, py)
      }
      ctx.stroke()
    }
  }

  const onResize = () => {
    if (!disposed) renderOnce(false)
  }
  window.addEventListener("resize", onResize, { passive: true })

  // Always draw a crisp initial architectural wireframe frame immediately on mount
  renderOnce(false)

  const drawLoop = () => {
    if (disposed || !isVisible || prefersReduced) return
    renderOnce(true)
    rafId = requestAnimationFrame(drawLoop)
  }

  const observer = new IntersectionObserver(
    (entries) => {
      const visibleNow = entries[0]?.isIntersecting ?? true
      if (visibleNow && !isVisible && !disposed) {
        isVisible = true
        if (!prefersReduced) rafId = requestAnimationFrame(drawLoop)
      } else if (!visibleNow) {
        isVisible = false
      }
    },
    { threshold: 0.02 }
  )
  observer.observe(canvas)

  if (!prefersReduced) {
    rafId = requestAnimationFrame(drawLoop)
  }

  return () => {
    disposed = true
    cancelAnimationFrame(rafId)
    observer.disconnect()
    parent?.removeEventListener("pointerenter", onEnter)
    parent?.removeEventListener("pointerleave", onLeave)
    window.removeEventListener("resize", onResize)
  }
}
