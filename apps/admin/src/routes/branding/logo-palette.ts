export type LogoPaletteSuggestion = {
  primary: string
  secondary: string
  accent: string
}

const MAX_SOURCE_LENGTH = 7 * 1024 * 1024
const MAX_IMAGE_BYTES = 5 * 1024 * 1024
const MAX_DIMENSION = 2048
const SAMPLE_EDGE = 64

export const extractLogoPalette = async (source: string): Promise<LogoPaletteSuggestion | null> => {
  if (!source || source.length > MAX_SOURCE_LENGTH) return null

  const isRasterDataUrl = /^data:image\/(png|jpeg|webp);base64,/i.test(source)
  if (source.startsWith("data:") && !isRasterDataUrl) return null
  let imageSource = source
  let temporaryObjectUrl: string | undefined
  if (!source.startsWith("data:")) {
    const controller = new AbortController()
    const timeoutId = window.setTimeout(() => controller.abort(), 5000)
    try {
      const parsed = new URL(source, window.location.origin)
      if (parsed.origin !== window.location.origin || !["http:", "https:"].includes(parsed.protocol)) return null
      const response = await fetch(parsed.href, { credentials: "same-origin", redirect: "error", signal: controller.signal })
      if (!response.ok) return null
      const contentType = response.headers.get("content-type")?.split(";")[0].trim().toLowerCase()
      if (!contentType || !["image/png", "image/jpeg", "image/webp"].includes(contentType)) return null
      const declaredLength = Number(response.headers.get("content-length"))
      if (Number.isFinite(declaredLength) && declaredLength > MAX_IMAGE_BYTES) return null
      if (!response.body) return null
      const reader = response.body.getReader()
      const chunks: ArrayBuffer[] = []
      let received = 0
      try {
        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          received += value.byteLength
          if (received > MAX_IMAGE_BYTES) {
            await reader.cancel()
            return null
          }
          const chunk = new ArrayBuffer(value.byteLength)
          new Uint8Array(chunk).set(value)
          chunks.push(chunk)
        }
      } finally {
        reader.releaseLock()
      }
      const blob = new Blob(chunks, { type: contentType })
      temporaryObjectUrl = URL.createObjectURL(blob)
      imageSource = temporaryObjectUrl
    } catch {
      return null
    } finally {
      window.clearTimeout(timeoutId)
    }
  }

  const image = new Image()
  let canvas: HTMLCanvasElement | undefined
  try {
    image.src = imageSource
    await image.decode()
    if (!image.naturalWidth || !image.naturalHeight || image.naturalWidth > MAX_DIMENSION || image.naturalHeight > MAX_DIMENSION) {
      return null
    }

    const scale = Math.min(1, SAMPLE_EDGE / Math.max(image.naturalWidth, image.naturalHeight))
    canvas = document.createElement("canvas")
    canvas.width = Math.max(1, Math.round(image.naturalWidth * scale))
    canvas.height = Math.max(1, Math.round(image.naturalHeight * scale))
    const context = canvas.getContext("2d", { willReadFrequently: true })
    if (!context) return null

    context.drawImage(image, 0, 0, canvas.width, canvas.height)
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data
    const histogram = new Map<string, number>()
    for (let index = 0; index < pixels.length; index += 4) {
      const alpha = pixels[index + 3]
      if (alpha < 220) continue
      const red = pixels[index]
      const green = pixels[index + 1]
      const blue = pixels[index + 2]
      const maximum = Math.max(red, green, blue)
      const minimum = Math.min(red, green, blue)
      const brightness = maximum / 255
      if (brightness < 0.10 || brightness > 0.96 || maximum - minimum < 28) continue
      const color = toHex(
        Math.round(red / 32) * 32,
        Math.round(green / 32) * 32,
        Math.round(blue / 32) * 32
      )
      histogram.set(color, (histogram.get(color) || 0) + 1)
    }

    const candidates = [...histogram.entries()]
      .sort(([colorA, countA], [colorB, countB]) => countB - countA || colorA.localeCompare(colorB))
      .map(([color]) => color)
    if (candidates.length === 0) return null
    const selected = candidates.slice(0, 3).map((color) => ensureReadableOnWhite(color))
    return {
      primary: selected[0],
      secondary: selected[1] || selected[0],
      accent: selected[2] || selected[1] || selected[0],
    }
  } catch {
    return null
  } finally {
    if (canvas) {
      canvas.width = 0
      canvas.height = 0
    }
    if (temporaryObjectUrl) URL.revokeObjectURL(temporaryObjectUrl)
  }
}

const toHex = (red: number, green: number, blue: number) =>
  `#${[red, green, blue].map((channel) => Math.min(channel, 255).toString(16).padStart(2, "0")).join("")}`

const ensureReadableOnWhite = (color: string) => {
  let [red, green, blue] = parseHex(color)
  for (let step = 0; step < 12 && contrastWithWhite(red, green, blue) < 4.5; step += 1) {
    red = Math.round(red * 0.82)
    green = Math.round(green * 0.82)
    blue = Math.round(blue * 0.82)
  }
  return toHex(red, green, blue)
}

const parseHex = (color: string): [number, number, number] => [
  Number.parseInt(color.slice(1, 3), 16),
  Number.parseInt(color.slice(3, 5), 16),
  Number.parseInt(color.slice(5, 7), 16),
]

const linearChannel = (value: number) => {
  const normalized = value / 255
  return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4
}

const contrastWithWhite = (red: number, green: number, blue: number) => {
  const luminance = 0.2126 * linearChannel(red) + 0.7152 * linearChannel(green) + 0.0722 * linearChannel(blue)
  return 1.05 / (luminance + 0.05)
}
