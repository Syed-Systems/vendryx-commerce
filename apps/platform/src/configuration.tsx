import React, { useEffect, useState } from "react"
import {
  Announcement,
  createAnnouncement,
  getAnnouncements,
  getFeatureFlags,
  getMessagingConfig,
  getPlatformDefaults,
  getThemeCatalog,
  GlobalFeatureFlag,
  MessagingConfig,
  PlatformDefaults,
  rolloutThemeVersion,
  ThemeCatalogItem,
  updateFeatureFlag,
  updateMessagingConfig,
  updatePlatformDefaults,
} from "./api"

export function ConfigurationView() {
  const [themes, setThemes] = useState<ThemeCatalogItem[]>([])
  const [flags, setFlags] = useState<GlobalFeatureFlag[]>([])
  const [defaults, setDefaults] = useState<PlatformDefaults | null>(null)
  const [messaging, setMessaging] = useState<MessagingConfig | null>(null)
  const [announcements, setAnnouncements] = useState<Announcement[]>([])

  // Forms
  const [annTitle, setAnnTitle] = useState("")
  const [annMessage, setAnnMessage] = useState("")
  const [annSeverity, setAnnSeverity] = useState<string>("info")
  const [annExpiryDays, setAnnExpiryDays] = useState(7)

  const [statusMessage, setStatusMessage] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    loadAll()
  }, [])

  const loadAll = () => {
    getThemeCatalog().then((res) => setThemes(res.themes))
    getFeatureFlags().then((res) => setFlags(res.feature_flags))
    getPlatformDefaults().then((res) => setDefaults(res.defaults))
    getMessagingConfig().then((res) => setMessaging(res.messaging))
    getAnnouncements().then((res) => setAnnouncements(res.announcements))
  }

  const handleToggleFlag = async (flag: GlobalFeatureFlag) => {
    try {
      const nextEnabled = !flag.is_enabled
      await updateFeatureFlag(flag.key, nextEnabled, nextEnabled ? 100 : 0)
      setStatusMessage(`Feature flag '${flag.key}' set to ${nextEnabled ? "ENABLED" : "DISABLED"}`)
      getFeatureFlags().then((res) => setFlags(res.feature_flags))
    } catch (err: any) {
      setStatusMessage(`Error updating flag: ${err.message}`)
    }
  }

  const handleSaveDefaults = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!defaults) return
    setLoading(true)
    try {
      await updatePlatformDefaults(defaults)
      setStatusMessage("Platform defaults saved successfully.")
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleSaveMessaging = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!messaging) return
    setLoading(true)
    try {
      await updateMessagingConfig(messaging)
      setStatusMessage("Messaging settings saved successfully.")
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateAnnouncement = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!annTitle.trim() || !annMessage.trim()) return
    setLoading(true)
    try {
      await createAnnouncement(annTitle.trim(), annMessage.trim(), annSeverity, annExpiryDays)
      setStatusMessage("Merchant announcement broadcast published.")
      setAnnTitle("")
      setAnnMessage("")
      getAnnouncements().then((res) => setAnnouncements(res.announcements))
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleRolloutTheme = async (themeId: string, version: string) => {
    try {
      await rolloutThemeVersion(themeId, version, 100)
      setStatusMessage(`Theme '${themeId}' staged rollout set to 100% on v${version}`)
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-2xl font-bold text-zinc-100 tracking-tight">Platform Configuration & Themes</h2>
        <p className="text-sm text-zinc-400 mt-1">
          Storefront theme catalog, feature-flag kill switches, region/tax defaults, and merchant broadcasts.
        </p>
      </div>

      {statusMessage && (
        <div className="p-3 bg-zinc-900 border border-zinc-700 rounded-lg text-xs font-mono text-zinc-200 flex items-center justify-between">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-zinc-500 hover:text-zinc-300">
            ✕
          </button>
        </div>
      )}

      {/* Grid: Theme Catalog & Feature Flags */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Theme Catalog */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-zinc-200">Theme Catalog & Rollouts</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-zinc-800 text-zinc-300">
              {themes.length} Shipped Themes
            </span>
          </div>

          <div className="space-y-3">
            {themes.map((t) => (
              <div key={t.id} className="p-4 bg-zinc-950 border border-zinc-800 rounded-lg space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-semibold text-zinc-200">{t.name}</span>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-900 text-zinc-400 border border-zinc-800">
                    v{t.latest_version}
                  </span>
                </div>
                <p className="text-xs text-zinc-400">{t.description}</p>
                {t.requires_tier_feature && (
                  <div className="text-[10px] text-indigo-400">Requires feature: {t.requires_tier_feature}</div>
                )}
                <div className="pt-2 flex items-center gap-2">
                  <button
                    onClick={() => handleRolloutTheme(t.id, t.latest_version)}
                    className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-[11px] font-medium text-zinc-200 rounded transition"
                  >
                    Staged Rollout v{t.latest_version}
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Global Feature Flags */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-zinc-200">Global Feature Flags</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-amber-950/60 text-amber-400 border border-amber-800/60">
              Deployment Wide
            </span>
          </div>

          <div className="space-y-3">
            {flags.map((f) => (
              <div key={f.key} className="p-4 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
                <div>
                  <div className="text-xs font-mono font-semibold text-zinc-200">{f.key}</div>
                  <div className="text-xs text-zinc-400">{f.description}</div>
                </div>
                <button
                  onClick={() => handleToggleFlag(f)}
                  className={`px-3 py-1.5 rounded text-xs font-semibold uppercase tracking-wider transition ${
                    f.is_enabled
                      ? "bg-emerald-950/80 text-emerald-400 border border-emerald-800"
                      : "bg-red-950/80 text-red-400 border border-red-800"
                  }`}
                >
                  {f.is_enabled ? "Enabled" : "Disabled"}
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Grid: Platform Defaults & Messaging */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Defaults */}
        {defaults && (
          <form onSubmit={handleSaveDefaults} className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
            <h3 className="text-base font-semibold text-zinc-200">Platform Defaults</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Default Region</label>
                <input
                  type="text"
                  value={defaults.default_region}
                  onChange={(e) => setDefaults({ ...defaults, default_region: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Default Currency</label>
                <input
                  type="text"
                  value={defaults.default_currency}
                  onChange={(e) => setDefaults({ ...defaults, default_currency: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
                />
              </div>
            </div>
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1">Default Tax Rate (e.g. 0.05)</label>
              <input
                type="number"
                step="0.01"
                value={defaults.default_tax_rate}
                onChange={(e) => setDefaults({ ...defaults, default_tax_rate: Number(e.target.value) })}
                className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
              />
            </div>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white rounded-lg transition"
            >
              Save Defaults
            </button>
          </form>
        )}

        {/* Messaging */}
        {messaging && (
          <form onSubmit={handleSaveMessaging} className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
            <h3 className="text-base font-semibold text-zinc-200">Email & SMS Messaging</h3>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Email Provider</label>
                <input
                  type="text"
                  value={messaging.email_provider}
                  onChange={(e) => setMessaging({ ...messaging, email_provider: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">SMS Provider</label>
                <input
                  type="text"
                  value={messaging.sms_provider}
                  onChange={(e) => setMessaging({ ...messaging, sms_provider: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
                />
              </div>
            </div>
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1">From Address</label>
              <input
                type="text"
                value={messaging.from_email}
                onChange={(e) => setMessaging({ ...messaging, from_email: e.target.value })}
                className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200"
              />
            </div>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white rounded-lg transition"
            >
              Save Messaging Config
            </button>
          </form>
        )}
      </div>

      {/* Broadcast Announcements */}
      <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-zinc-200">Merchant Dashboard Broadcasts</h3>
          <span className="text-xs text-zinc-400">{announcements.length} Published Broadcasts</span>
        </div>

        <form onSubmit={handleCreateAnnouncement} className="grid grid-cols-1 sm:grid-cols-4 gap-3">
          <input
            type="text"
            placeholder="Announcement Title"
            value={annTitle}
            onChange={(e) => setAnnTitle(e.target.value)}
            className="px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200"
          />
          <input
            type="text"
            placeholder="Broadcast Message"
            value={annMessage}
            onChange={(e) => setAnnMessage(e.target.value)}
            className="px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 sm:col-span-2"
          />
          <button
            type="submit"
            disabled={loading || !annTitle.trim() || !annMessage.trim()}
            className="py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-xs font-semibold text-white rounded-lg transition"
          >
            Publish Broadcast
          </button>
        </form>

        <div className="space-y-2 pt-2">
          {announcements.map((a) => (
            <div key={a.id} className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-zinc-200">{a.title}</div>
                <div className="text-xs text-zinc-400">{a.message}</div>
              </div>
              <div className="text-right">
                <span className="text-[10px] uppercase font-bold px-2 py-0.5 rounded bg-zinc-900 text-zinc-400 border border-zinc-800">
                  {a.severity}
                </span>
                <div className="text-[10px] text-zinc-500 mt-1">
                  Expires: {new Date(a.expires_at).toLocaleDateString()}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
