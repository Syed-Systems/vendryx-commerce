import React, { useEffect, useState } from "react"
import {
  ChurnCohort,
  FunnelMetrics,
  getChurnCohorts,
  getFunnelMetrics,
  getGmvMetrics,
  getPlanDistribution,
  getQuotaHeadroom,
  getTopMerchants,
  GmvMetrics,
  PlanDistributionItem,
  QuotaHeadroom,
  TopMerchantGmv,
} from "./api"

export function AnalyticsView() {
  const [gmv, setGmv] = useState<GmvMetrics | null>(null)
  const [funnel, setFunnel] = useState<FunnelMetrics | null>(null)
  const [distribution, setDistribution] = useState<PlanDistributionItem[]>([])
  const [topMerchants, setTopMerchants] = useState<TopMerchantGmv[]>([])
  const [cohorts, setCohorts] = useState<ChurnCohort[]>([])
  const [headroom, setHeadroom] = useState<QuotaHeadroom[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadData = async () => {
    setLoading(true)
    setError(null)
    try {
      const [g, f, d, t, c, h] = await Promise.all([
        getGmvMetrics().catch(() => ({ gmv_metrics: null })),
        getFunnelMetrics().catch(() => ({ funnel: null })),
        getPlanDistribution().catch(() => ({ distribution: [] })),
        getTopMerchants(5).catch(() => ({ top_merchants: [] })),
        getChurnCohorts().catch(() => ({ cohorts: [] })),
        getQuotaHeadroom().catch(() => ({ headroom: [] })),
      ])
      setGmv(g.gmv_metrics)
      setFunnel(f.funnel)
      setDistribution(d.distribution)
      setTopMerchants(t.top_merchants)
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
      <div className="flex items-center justify-center p-12 text-zinc-400">
        <span className="animate-spin mr-3">⟳</span> Loading business telemetry...
      </div>
    )
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-zinc-100 tracking-tight">Growth & Revenue Analytics</h2>
          <p className="text-sm text-zinc-400 mt-1">
            Real-time cross-tenant business metrics, cohort retention, and merchant headroom.
          </p>
        </div>
        <button
          onClick={loadData}
          className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-200 rounded border border-zinc-700 transition"
        >
          Refresh Data
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-950/40 border border-red-800/60 rounded-lg text-red-300 text-sm">
          {error}
        </div>
      )}

      {/* Top Metrics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="p-5 bg-zinc-900/60 border border-zinc-800 rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-zinc-400">Total Platform GMV</span>
          <div className="text-2xl font-bold text-emerald-400">
            {gmv ? `${gmv.total_gmv.currency} ${gmv.total_gmv.amount}` : "USD 0.00"}
          </div>
          <span className="text-xs text-zinc-500">All-time settled volume</span>
        </div>

        <div className="p-5 bg-zinc-900/60 border border-zinc-800 rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-zinc-400">30-Day GMV</span>
          <div className="text-2xl font-bold text-zinc-100">
            {gmv ? `${gmv.monthly_gmv.currency} ${gmv.monthly_gmv.amount}` : "USD 0.00"}
          </div>
          <span className="text-xs text-zinc-500">Trailing 30 days</span>
        </div>

        <div className="p-5 bg-zinc-900/60 border border-zinc-800 rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-zinc-400">Activation Rate</span>
          <div className="text-2xl font-bold text-cyan-400">
            {funnel ? `${(funnel.activation_rate * 100).toFixed(1)}%` : "0.0%"}
          </div>
          <span className="text-xs text-zinc-500">
            {funnel ? `${funnel.activated_merchants} of ${funnel.total_signups} signups` : "0 signups"}
          </span>
        </div>

        <div className="p-5 bg-zinc-900/60 border border-zinc-800 rounded-xl space-y-1">
          <span className="text-xs uppercase font-semibold tracking-wider text-zinc-400">Paid Conversion</span>
          <div className="text-2xl font-bold text-indigo-400">
            {funnel ? `${(funnel.paid_conversion_rate * 100).toFixed(1)}%` : "0.0%"}
          </div>
          <span className="text-xs text-zinc-500">
            {funnel ? `${funnel.paid_subscribers} paid subscribers` : "0 paid"}
          </span>
        </div>
      </div>

      {/* Funnel & Plan Distribution */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Onboarding & Conversion Funnel */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-zinc-200">Merchant Conversion Funnel</h3>
          <div className="space-y-3">
            <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-zinc-400 uppercase">1. Signup</div>
                <div className="text-sm font-medium text-zinc-200">{funnel?.total_signups || 0} Registered Merchants</div>
              </div>
              <span className="text-xs font-bold text-zinc-400">100%</span>
            </div>

            <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-cyan-400 uppercase">2. Activated</div>
                <div className="text-sm font-medium text-zinc-200">{funnel?.activated_merchants || 0} Active Merchants</div>
              </div>
              <span className="text-xs font-bold text-cyan-400">
                {funnel ? `${(funnel.activation_rate * 100).toFixed(1)}%` : "0%"}
              </span>
            </div>

            <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-indigo-400 uppercase">3. Paid Tier</div>
                <div className="text-sm font-medium text-zinc-200">{funnel?.paid_subscribers || 0} Paid Subscribers</div>
              </div>
              <span className="text-xs font-bold text-indigo-400">
                {funnel ? `${(funnel.paid_conversion_rate * 100).toFixed(1)}%` : "0%"}
              </span>
            </div>
          </div>
        </div>

        {/* Plan Distribution */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-zinc-200">Subscription Tier Distribution</h3>
          <div className="space-y-2">
            {distribution.length === 0 ? (
              <div className="text-xs text-zinc-500 italic p-4 text-center">No active tiers configured.</div>
            ) : (
              distribution.map((item) => (
                <div key={item.tier} className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg space-y-1">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-semibold text-zinc-300 uppercase">{item.tier}</span>
                    <span className="text-zinc-400">
                      {item.count} merchants ({item.percentage.toFixed(1)}%)
                    </span>
                  </div>
                  <div className="w-full bg-zinc-800 rounded-full h-1.5 overflow-hidden">
                    <div
                      className="bg-indigo-500 h-1.5 rounded-full"
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
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-zinc-200">Monthly Retention & Churn</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="text-zinc-500 uppercase border-b border-zinc-800">
                <tr>
                  <th className="pb-2">Cohort</th>
                  <th className="pb-2">Initial</th>
                  <th className="pb-2">Active</th>
                  <th className="pb-2">Churned</th>
                  <th className="pb-2">Retention</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
                {cohorts.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="py-4 text-center text-zinc-500 italic">
                      No cohort data recorded yet.
                    </td>
                  </tr>
                ) : (
                  cohorts.map((c) => (
                    <tr key={c.signup_month}>
                      <td className="py-2.5 font-medium text-zinc-200">{c.signup_month}</td>
                      <td className="py-2.5">{c.initial_count}</td>
                      <td className="py-2.5 text-emerald-400">{c.active_count}</td>
                      <td className="py-2.5 text-red-400">{c.churned_count}</td>
                      <td className="py-2.5 font-semibold text-indigo-400">
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
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <h3 className="text-base font-semibold text-zinc-200">Upsell Radar: Quota Headroom</h3>
          <div className="space-y-2">
            {headroom.length === 0 ? (
              <div className="text-xs text-zinc-500 italic p-4 text-center">
                All merchants currently within safe quota margins.
              </div>
            ) : (
              headroom.map((h) => (
                <div
                  key={`${h.tenant_id}-${h.metric}`}
                  className="p-3 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between"
                >
                  <div>
                    <div className="text-xs font-semibold text-zinc-200">{h.name}</div>
                    <div className="text-xs text-zinc-400">
                      {h.metric}: {h.usage} / {h.limit} ({h.usage_percentage.toFixed(0)}%)
                    </div>
                  </div>
                  <span className="px-2 py-0.5 text-[10px] font-bold uppercase rounded bg-amber-950/60 text-amber-400 border border-amber-800/60">
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
