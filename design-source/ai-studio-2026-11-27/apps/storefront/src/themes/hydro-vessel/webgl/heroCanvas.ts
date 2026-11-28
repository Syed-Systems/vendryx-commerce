// Interactive WebGL2 Topographic Contour Field & Subtle Alpine Pointer Glow
// Scoped strictly to its theme root (`data-vessel-mode`) so multi-tenant styling never leaks globally.
export function initHeroCanvas(
  canvas: HTMLCanvasElement | null,
  getMode: () => "light" | "dark"
): () => void {
  if (!canvas) return () => {}

  const gl = canvas.getContext("webgl2", {
    alpha: false,
    antialias: false,
    powerPreference: "low-power",
  })
  if (!gl) return () => {}

  // Detect software WebGL (SwiftShader / llvmpipe) to cap pixel ratio & pace rendering gently
  const isSoftwareGl = (() => {
    try {
      const ext = gl.getExtension("WEBGL_debug_renderer_info")
      const param = ext ? ext.UNMASKED_RENDERER_WEBGL : gl.RENDERER
      const rendererStr = String(gl.getParameter(param) ?? "")
      return /SwiftShader|llvmpipe|softpipe|Software|Basic Render/i.test(rendererStr)
    } catch {
      return false
    }
  })()

  const vsSource = `#version 300 es
  in vec2 a_pos;
  out vec2 v_uv;
  void main() {
    v_uv = a_pos * 0.5 + 0.5;
    gl_Position = vec4(a_pos, 0.0, 1.0);
  }`

  const fsSource = `#version 300 es
  precision highp float;
  in vec2 v_uv;
  out vec4 outColor;
  uniform float u_time;
  uniform vec2 u_mouse;
  uniform vec2 u_res;
  uniform float u_isDark;

  float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
  }

  float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
  }

  void main() {
    vec2 uv = v_uv;
    float aspect = u_res.x / max(u_res.y, 1.0);
    vec2 p = (uv - 0.5) * vec2(aspect, 1.0);

    float n1 = noise(p * 2.4 + vec2(u_time * 0.04, -u_time * 0.025));
    float n2 = noise(p * 5.2 - vec2(u_time * 0.03, u_time * 0.02));
    float elev = n1 * 0.65 + n2 * 0.35;
    float contour = abs(fract(elev * 9.0) - 0.5);
    float line = smoothstep(0.47, 0.495, contour);

    vec2 mPos = (u_mouse - 0.5) * vec2(aspect, 1.0);
    float dist = length(p - mPos);
    float glow = exp(-dist * dist * 2.4);

    vec3 darkBg = vec3(0.047, 0.063, 0.055);
    vec3 darkAccent = vec3(0.282, 0.388, 0.294);
    vec3 lightBg = vec3(0.953, 0.965, 0.957);
    vec3 lightAccent = vec3(0.239, 0.341, 0.251);

    vec3 baseCol = mix(lightBg, darkBg, u_isDark);
    vec3 sageCol = mix(lightAccent, darkAccent, u_isDark);
    vec3 col = baseCol;
    col = mix(col, sageCol, glow * (u_isDark > 0.5 ? 0.26 : 0.14));
    col += sageCol * line * (u_isDark > 0.5 ? 0.09 : 0.065);
    outColor = vec4(col, 1.0);
  }`

  const compile = (type: number, src: string): WebGLShader | null => {
    const sh = gl.createShader(type)
    if (!sh) return null
    gl.shaderSource(sh, src)
    gl.compileShader(sh)
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
      gl.deleteShader(sh)
      return null
    }
    return sh
  }

  const prog = gl.createProgram()
  if (!prog) return () => {}
  const vs = compile(gl.VERTEX_SHADER, vsSource)
  const fs = compile(gl.FRAGMENT_SHADER, fsSource)
  if (!vs || !fs) {
    if (vs) gl.deleteShader(vs)
    if (fs) gl.deleteShader(fs)
    gl.deleteProgram(prog)
    return () => {}
  }

  gl.attachShader(prog, vs)
  gl.attachShader(prog, fs)
  gl.linkProgram(prog)
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    gl.deleteShader(vs)
    gl.deleteShader(fs)
    gl.deleteProgram(prog)
    return () => {}
  }

  const buf = gl.createBuffer()
  gl.bindBuffer(gl.ARRAY_BUFFER, buf)
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW)

  const aPos = gl.getAttribLocation(prog, "a_pos")
  const uTime = gl.getUniformLocation(prog, "u_time")
  const uMouse = gl.getUniformLocation(prog, "u_mouse")
  const uRes = gl.getUniformLocation(prog, "u_res")
  const uIsDark = gl.getUniformLocation(prog, "u_isDark")

  let mouseX = 0.72
  let mouseY = 0.5
  let targetX = 0.72
  let targetY = 0.5
  let rafId = 0
  let timerId = 0
  let isVisible = true
  let disposed = false

  const resize = () => {
    const dpr = isSoftwareGl ? 1 : Math.min(window.devicePixelRatio || 1, 2)
    const w = Math.max(1, Math.floor(canvas.clientWidth * dpr))
    const h = Math.max(1, Math.floor(canvas.clientHeight * dpr))
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w
      canvas.height = h
    }
  }

  const onPointerMove = (e: PointerEvent) => {
    const rect = canvas.getBoundingClientRect()
    if (!rect.width || !rect.height) return
    targetX = (e.clientX - rect.left) / rect.width
    targetY = 1.0 - (e.clientY - rect.top) / rect.height
  }

  const observer = new IntersectionObserver(
    (entries) => {
      const visibleNow = entries[0]?.isIntersecting ?? true
      if (visibleNow && !isVisible && !disposed) {
        isVisible = true
        scheduleNext()
      } else {
        isVisible = visibleNow
      }
    },
    { threshold: 0.05 }
  )
  observer.observe(canvas)

  window.addEventListener("pointermove", onPointerMove, { passive: true })
  window.addEventListener("resize", resize, { passive: true })
  resize()

  const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches
  const start = performance.now()

  const drawFrame = (now: number) => {
    if (disposed) return
    resize()
    mouseX += (targetX - mouseX) * 0.08
    mouseY += (targetY - mouseY) * 0.08
    const isDark = getMode() === "dark" ? 1.0 : 0.0
    const t = prefersReduced ? 0.0 : (now - start) * 0.001

    gl.viewport(0, 0, canvas.width, canvas.height)
    gl.useProgram(prog)
    gl.enableVertexAttribArray(aPos)
    gl.bindBuffer(gl.ARRAY_BUFFER, buf)
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0)
    gl.uniform1f(uTime, t)
    gl.uniform2f(uMouse, mouseX, mouseY)
    gl.uniform2f(uRes, canvas.width, canvas.height)
    gl.uniform1f(uIsDark, isDark)
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4)

    if (!prefersReduced && isVisible) {
      scheduleNext()
    }
  }

  const scheduleNext = () => {
    if (isSoftwareGl) {
      // Gentle pacing under software WebGL so CPU / headless browser never stalls
      timerId = window.setTimeout(() => {
        rafId = requestAnimationFrame(drawFrame)
      }, 120)
    } else {
      rafId = requestAnimationFrame(drawFrame)
    }
  }

  rafId = requestAnimationFrame(drawFrame)

  return () => {
    disposed = true
    cancelAnimationFrame(rafId)
    window.clearTimeout(timerId)
    observer.disconnect()
    window.removeEventListener("pointermove", onPointerMove)
    window.removeEventListener("resize", resize)
    gl.deleteBuffer(buf)
    gl.deleteShader(vs)
    gl.deleteShader(fs)
    gl.deleteProgram(prog)
  }
}
