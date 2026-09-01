import { Line } from "@react-three/drei"
import { Canvas, useFrame } from "@react-three/fiber"
import { useMotionValueEvent, useReducedMotion, useScroll, type MotionValue } from "motion/react"
import { useMemo, useRef, useState } from "react"
import type { Group, Mesh } from "three"
import { Vector3 } from "three"

const ACCENT = "#1c8b82"
const RADIUS = 1.62

type Country = {
  name: string
  region: string
  position: [number, number]
  outline: [number, number][]
}

const countries: Country[] = [
  { name: "Bangladesh", region: "South Asia", position: [23.7, 90.4], outline: [[26.6, 88.1], [26.2, 89.3], [26.0, 90.6], [25.7, 91.4], [25.1, 92.1], [24.2, 92.4], [23.2, 91.7], [22.5, 91.8], [21.1, 91.9], [20.8, 90.3], [21.8, 89.2], [22.7, 88.4], [24.2, 88.0], [25.4, 88.2], [26.6, 88.1]] },
  { name: "United Arab Emirates", region: "Gulf", position: [24.4, 54.4], outline: [[26.1, 51.5], [26.1, 56.4], [25.5, 56.4], [24.8, 55.9], [24.2, 55.8], [23.1, 55.3], [22.7, 55.2], [22.6, 52.6], [24.0, 51.6], [26.1, 51.5]] },
  { name: "United Kingdom", region: "Europe", position: [54.2, -2.8], outline: [[58.7, -3.3], [58.1, -5.7], [57.2, -5.0], [56.2, -5.8], [55.3, -4.7], [54.5, -5.8], [53.4, -4.8], [52.9, -3.0], [51.1, -1.3], [50.2, -4.7], [51.0, -5.6], [52.2, -5.0], [53.2, -3.4], [54.3, -3.1], [55.2, -2.0], [56.2, -2.3], [57.1, -3.5], [58.0, -3.2], [58.7, -3.3]] },
  { name: "United States", region: "North America", position: [39.8, -98.6], outline: [[49.0, -124.8], [48.7, -123.0], [46.0, -123.2], [42.0, -124.4], [38.0, -122.5], [34.5, -120.5], [32.5, -117.2], [31.4, -111.0], [29.0, -104.7], [25.8, -97.2], [28.8, -82.8], [30.5, -81.1], [35.0, -79.0], [36.5, -75.7], [40.0, -74.0], [43.5, -70.3], [45.0, -68.8], [47.5, -69.2], [49.0, -95.0], [49.0, -124.8]] },
]

function globePoint([lat, lon]: [number, number], radius = RADIUS) {
  const phi = ((90 - lat) * Math.PI) / 180
  const theta = ((lon + 180) * Math.PI) / 180
  return new Vector3(
    -radius * Math.sin(phi) * Math.cos(theta),
    radius * Math.cos(phi),
    radius * Math.sin(phi) * Math.sin(theta),
  )
}

function GlobeGrid() {
  const lines = useMemo(() => {
    const latitude = [-60, -30, 0, 30, 60].map((lat) =>
      Array.from({ length: 73 }, (_, index) => globePoint([lat, -180 + index * 5], RADIUS + 0.008)),
    )
    const longitude = [-150, -120, -90, -60, -30, 0, 30, 60, 90, 120, 150].map((lon) =>
      Array.from({ length: 37 }, (_, index) => globePoint([-90 + index * 5, lon], RADIUS + 0.008)),
    )
    return [...latitude, ...longitude]
  }, [])
  return (
    <>
      {lines.map((points, index) => (
        <Line key={index} points={points} color="#8a857d" transparent opacity={0.16} lineWidth={0.65} />
      ))}
    </>
  )
}

function CountryOutline({ country, selected }: { country: Country; selected: boolean }) {
  const points = useMemo(() => country.outline.map((point) => globePoint(point, RADIUS + 0.024)), [country])
  const marker = useMemo(() => globePoint(country.position, RADIUS + 0.04), [country])
  return (
    <>
      <Line
        points={points}
        color={selected ? ACCENT : "#a8a29e"}
        transparent
        opacity={selected ? 0.95 : 0.45}
        lineWidth={selected ? 2.2 : 0.9}
      />
      {selected && (
        <mesh position={marker}>
          <sphereGeometry args={[0.048, 16, 16]} />
          <meshBasicMaterial color={ACCENT} />
        </mesh>
      )}
    </>
  )
}

function MerchantGlobe({ progress, active, reduceMotion }: { progress: MotionValue<number>; active: number; reduceMotion: boolean }) {
  const rig = useRef<Group>(null)
  const planet = useRef<Mesh>(null)
  const targetRotation = useRef(0.16)

  useFrame(() => {
    const progressValue = reduceMotion ? 0 : progress.get()
    targetRotation.current = 0.16 + progressValue * Math.PI * 1.25
    if (rig.current) {
      rig.current.rotation.y += (targetRotation.current - rig.current.rotation.y) * 0.035
      rig.current.rotation.x = -0.22 + Math.sin(progressValue * Math.PI) * 0.15
      rig.current.position.y = 0.2 - progressValue * 0.32
    }
    if (planet.current) planet.current.rotation.y -= reduceMotion ? 0 : 0.0006
  })

  return (
    <group ref={rig}>
      <mesh ref={planet}>
        <sphereGeometry args={[RADIUS, 64, 64]} />
        <meshStandardMaterial color="#211f1c" roughness={0.6} metalness={0.15} emissive="#0d0c0b" emissiveIntensity={0.1} />
      </mesh>
      <GlobeGrid />
      {countries.map((country, index) => (
        <CountryOutline key={country.name} country={country} selected={index === active} />
      ))}
    </group>
  )
}

export function ScrollScene() {
  const { scrollYProgress } = useScroll()
  const reduceMotion = Boolean(useReducedMotion())
  const [active, setActive] = useState(0)

  useMotionValueEvent(scrollYProgress, "change", (value) => {
    if (reduceMotion) return
    const next = Math.min(countries.length - 1, Math.floor(value * countries.length))
    setActive((current) => (current === next ? current : next))
  })
  const country = countries[active]

  return (
    <div className="scroll-scene" aria-hidden="true">
      <Canvas dpr={[1, 1.5]} camera={{ position: [0, 0, 7.5], fov: 38 }} gl={{ antialias: true, alpha: true }}>
        <ambientLight intensity={0.7} />
        <directionalLight position={[4, 5, 6]} intensity={2.0} color="#fff6eb" />
        <pointLight position={[-3, -2, 2]} intensity={12} distance={7} color={ACCENT} />
        <MerchantGlobe progress={scrollYProgress} active={active} reduceMotion={reduceMotion} />
      </Canvas>
      <div className="globe-country-card">
        <span className="globe-badge-label">
          <span className="globe-badge-dot" />
          Merchant Network
        </span>
        <strong>{country.name}</strong>
        <small>{country.region}</small>
      </div>
    </div>
  )
}
