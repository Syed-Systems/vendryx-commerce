import { useEffect, useState } from "react"
import {
  ChurnCohort,
  FunnelMetrics,
  getChurnCohorts,
  getFunnelMetrics,
  getGmvMetrics,
  getPlanDistribution,
  getQuotaHeadroom,
  GmvMetrics,
  PlanDistributionItem,
  QuotaHeadroom,
} from "./api"

export function AnalyticsView() {
  const [gmv, setGmv] = useState<GmvMetrics | null>(null)
  const [funnel, setFunnel] = useState<FunnelMetrics | null>(null)
  const [distribution, setDistribution] = useState<PlanDistributionItem[]>([])
  const [cohorts, setCohorts] = useState<ChurnCohort[]>([])
  const [headroom, setHeadroom] = useState<QuotaHeadroom[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [g, f, d, c, h] = await Promise.all([
        getGmvMetrics().catch(() => ({ gmv_metrics: null })),
        getFunnelMetrics().catch(() => ({ funnel: null })),
        getPlanDistribution().catch(() => ({ distribution: [] })),
        getChurnCohorts().catch(() => ({ cohorts: [] })),
        getQuotaHeadroom().catch(() => ({ headroom: [] })),
      ])
      setGmv(g.gmv_metrics)
      setFunnel(f.funnel)
      setDistribution(d.distribution)
      setCohorts(c.cohorts)
      setHeadroom(h.headroom)
    } catch (e: any) {
      setError(e?.message || "Failed to load growth analytics")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadData()
  }, [])

  if (loading) {
    return (
      <div className="flex items-center justify-center p-12 text-ui-fg-subtle">
        <span className="animate-spin mr-3">⟳</span> Loading business telemetry...
      </div>
    )
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-ui-fg-base tracking-tight">Growth & Revenue Analytics</h2>
          <p className="text-sm text-ui-fg-subtle mt-1">
            Real-time cross-tenant business metrics, cohort retention, and merchant headroom.
          </p>
        </div>
        <button
          onClick={loadData}
          className="px-3 py-1.5 bg-ui-bg-component hover:bg-ui-bg-component-hover text-xs font-medium text-ui-fg-base rounded border border-ui-border-strong transition"
        >
          Refresh Data
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-800/60 rounded-lg text-red-700 dark:text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Top Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-ui-fg-subtle">Total Platform GMV</span>
          <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
            {gmv ? `${gmv.total_gmv.currency} ${gmv.total_gmv.amount}` : "USD 0.00"}
          </div>
          <span className="text-xs text-ui-fg-muted">All-time settled volume</span>
        </div>

        <div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-ui-fg-subtle">30-Day GMV</span>
          <div className="text-2xl font-bold text-ui-fg-base">
            {gmv ? `${gmv.monthly_gmv.currency} ${gmv.monthly_gmv.amount}` : "USD 0.00"}
          </div>
          <span className="text-xs text-ui-fg-muted">Trailing 30 days</span>
        </div>

        <div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-ui-fg-subtle">Activation Rate</span>
          <div className="text-2xl font-bold text-cyan-600 dark:text-cyan-400">
            {funnel ? `${(funnel.activation_rate * 100).toFixed(1)}%` : "0.0%"}
          </div>
          <span className="text-xs text-ui-fg-muted">
            {funnel ? `${funnel.activated_merchants} of ${funnel.total_signups} signups` : "0 signups"}
          </span>
        </div>

        <div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-ui-fg-subtle">Paid Conversion</span>
          <div className="text-2xl font-bold text-ui-fg-interactive">
            {funnel ? `${(funnel.paid_conversion_rate * 100).toFixed(1)}%` : "0.0%"}
          </div>
          <span className="text-xs text-ui-fg-muted">
            {funnel ? `${funnel.paid_subscribers} paid subscribers` : "0 paid"}
          </span>
        </div>
      </div>

      {/* Funnel & Plan Distribution */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Onboarding & Conversion Funnel */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-ui-fg-base">Merchant Conversion Funnel</h3>
          <div className="space-y-3">
            <div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-ui-fg-subtle uppercase">1. Signup</div>
                <div className="text-sm font-medium text-ui-fg-base">{funnel?.total_signups || 0} Registered Merchants</div>
              </div>
              <span className="text-xs font-bold text-ui-fg-subtle">100%</span>
            </div>

            <div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-cyan-600 dark:text-cyan-400 uppercase">2. Activated</div>
                <div className="text-sm font-medium text-ui-fg-base">{funnel?.activated_merchants || 0} Active Merchants</div>
              </div>
              <span className="text-xs font-bold text-cyan-600 dark:text-cyan-400">
                {funnel ? `${(funnel.activation_rate * 100).toFixed(1)}%` : "0%"}
              </span>
            </div>

            <div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-ui-fg-interactive uppercase">3. Paid Tier</div>
                <div className="text-sm font-medium text-ui-fg-base">{funnel?.paid_subscribers || 0} Paid Subscribers</div>
              </div>
              <span className="text-xs font-bold text-ui-fg-interactive">
                {funnel ? `${(funnel.paid_conversion_rate * 100).toFixed(1)}%` : "0%"}
              </span>
            </div>
          </div>
        </div>

        {/* Plan Distribution */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-ui-fg-base">Subscription Tier Distribution</h3>
          <div className="space-y-2">
            {distribution.length === 0 ? (
              <div className="text-xs text-ui-fg-muted italic p-4 text-center">No active tiers configured.</div>
            ) : (
              distribution.map((item) => (
                <div key={item.tier} className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg space-y-1">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-semibold text-ui-fg-subtle uppercase">{item.tier}</span>
                    <span className="text-ui-fg-subtle">
                      {item.count} merchants ({item.percentage.toFixed(1)}%)
                    </span>
                  </div>
                  <div className="w-full bg-ui-bg-component rounded-full h-1.5 overflow-hidden">
                    <div
                      className="bg-ui-bg-interactive h-1.5 rounded-full"
                      style={{ width: `${Math.min(100, item.percentage)}%` }}
                    />
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Cohorts & Headroom Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Retention Cohorts */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-ui-fg-base">Monthly Retention & Churn</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="text-ui-fg-muted uppercase border-b border-ui-border-base">
                <tr>
                  <th className="pb-2">Cohort</th>
                  <th className="pb-2">Initial</th>
                  <th className="pb-2">Active</th>
                  <th className="pb-2">Churned</th>
                  <th className="pb-2">Retention</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-ui-border-base text-ui-fg-subtle">
                {cohorts.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="py-4 text-center text-ui-fg-muted italic">
                      No cohort data recorded yet.
                    </td>
                  </tr>
                ) : (
                  cohorts.map((c) => (
                    <tr key={c.signup_month}>
                      <td className="py-2.5 font-medium text-ui-fg-base">{c.signup_month}</td>
                      <td className="py-2.5">{c.initial_count}</td>
                      <td className="py-2.5 text-emerald-600 dark:text-emerald-400">{c.active_count}</td>
                      <td className="py-2.5 text-red-600 dark:text-red-400">{c.churned_count}</td>
                      <td className="py-2.5 font-semibold text-ui-fg-interactive">
                        {c.retention_rate.toFixed(1)}%
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>

        {/* Quota Headroom & Upsell Signals */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-ui-fg-base">Upsell Radar: Quota Headroom</h3>
          <div className="space-y-2">
            {headroom.length === 0 ? (
              <div className="text-xs text-ui-fg-muted italic p-4 text-center">
                All merchants currently within safe quota margins.
              </div>
            ) : (
              headroom.map((h) => (
                <div
                  key={`${h.tenant_id}-${h.metric}`}
                  className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg flex items-center justify-between"
                >
                  <div>
                    <div className="text-xs font-semibold text-ui-fg-base">{h.name}</div>
                    <div className="text-xs text-ui-fg-subtle">
                      {h.metric}: {h.usage} / {h.limit} ({h.usage_percentage.toFixed(0)}%)
                    </div>
                  </div>
                  <span className="px-2 py-0.5 text-[10px] font-bold uppercase rounded bg-amber-50 dark:bg-amber-950/60 text-amber-600 dark:text-amber-400 border border-amber-200 dark:border-amber-800/60">
                    Upsell Target
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
