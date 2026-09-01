import React, { useEffect, useState } from "react"
import {
  endImpersonation,
  ImpersonationResponse,
  listTenants,
  listWorkflows,
  startImpersonation,
  Tenant,
  toggleTenantReadOnly,
  triggerManualJob,
  WorkflowExecution,
} from "./api"

export function SupportView() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [selectedTenant, setSelectedTenant] = useState<string>("")
  const [reason, setReason] = useState("")
  const [duration, setDuration] = useState(30)
  const [activeSession, setActiveSession] = useState<ImpersonationResponse | null>(null)
  const [workflows, setWorkflows] = useState<WorkflowExecution[]>([])
  const [isReadOnly, setIsReadOnly] = useState(false)
  const [readOnlyReason, setReadOnlyReason] = useState("")
  const [statusMessage, setStatusMessage] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    listTenants().then((res) => {
      setTenants(res.tenants)
      if (res.tenants.length > 0) {
        setSelectedTenant(res.tenants[0].id)
      }
    })
    listWorkflows().then((res) => setWorkflows(res.workflows))
  }, [])

  const handleStartImpersonate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!selectedTenant || !reason.trim()) return
    setLoading(true)
    try {
      const res = await startImpersonation(selectedTenant, {
        duration_minutes: duration,
        reason: reason.trim(),
      })
      setActiveSession(res.impersonation)
      setStatusMessage(`Impersonation session active for ${selectedTenant}`)
      setReason("")
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleEndImpersonate = async () => {
    if (!activeSession) return
    setLoading(true)
    try {
      await endImpersonation(activeSession.tenant_id, activeSession.token)
      setActiveSession(null)
      setStatusMessage("Impersonation session terminated safely.")
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleTriggerJob = async (jobName: string) => {
    setLoading(true)
    try {
      const res = await triggerManualJob(jobName, selectedTenant || undefined)
      setStatusMessage(`Job '${res.job_name}' queued successfully.`)
    } catch (err: any) {
      setStatusMessage(`Job Trigger Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  const handleToggleReadOnly = async () => {
    if (!selectedTenant || !readOnlyReason.trim()) return
    setLoading(true)
    try {
      const nextState = !isReadOnly
      await toggleTenantReadOnly(selectedTenant, nextState, readOnlyReason.trim())
      setIsReadOnly(nextState)
      setStatusMessage(`Tenant ${selectedTenant} is now ${nextState ? "READ-ONLY" : "WRITABLE"}.`)
      setReadOnlyReason("")
    } catch (err: any) {
      setStatusMessage(`Error: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold text-ui-fg-base tracking-tight">Support & Incident Operations</h2>
        <p className="text-sm text-ui-fg-subtle mt-1">
          Time-boxed merchant impersonation, emergency maintenance locks, and manual system job dispatch.
        </p>
      </div>

      {statusMessage && (
        <div className="p-3 bg-ui-bg-component border border-ui-border-strong rounded-lg text-xs font-mono text-ui-fg-base flex items-center justify-between">
          <span>{statusMessage}</span>
          <button onClick={() => setStatusMessage(null)} className="text-ui-fg-muted hover:text-ui-fg-base">
            ✕
          </button>
        </div>
      )}

      {/* Grid: Impersonation & Emergency Locks */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Support Impersonation */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-ui-fg-base">Merchant Impersonation</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-ui-bg-highlight text-ui-fg-interactive border border-ui-border-interactive">
              ADR-018 Audited
            </span>
          </div>
          <p className="text-xs text-ui-fg-subtle">
            Assume merchant scope with server-side time boxing. Secrets and billing modification are strictly barred.
          </p>

          {activeSession ? (
            <div className="p-4 bg-ui-bg-subtle border border-ui-border-interactive rounded-lg space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-ui-fg-interactive">Active Support Session</span>
                <span className="text-xs text-ui-fg-subtle font-mono">
                  Expires: {new Date(activeSession.expires_at).toLocaleTimeString()}
                </span>
              </div>
              <div className="p-2 bg-ui-bg-component rounded font-mono text-xs text-ui-fg-subtle truncate">
                {activeSession.token}
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => navigator.clipboard.writeText(activeSession.token)}
                  className="px-3 py-1.5 bg-ui-bg-component hover:bg-ui-bg-component-hover text-xs font-medium text-ui-fg-base rounded transition"
                >
                  Copy Session Token
                </button>
                <button
                  onClick={handleEndImpersonate}
                  className="px-3 py-1.5 bg-red-50 dark:bg-red-950/80 hover:bg-red-100 dark:hover:bg-red-900 text-xs font-medium text-red-700 dark:text-red-300 rounded border border-red-200 dark:border-red-800 transition"
                >
                  End Session Now
                </button>
              </div>
            </div>
          ) : (
            <form onSubmit={handleStartImpersonate} className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-ui-fg-subtle mb-1">Select Tenant</label>
                <select
                  value={selectedTenant}
                  onChange={(e) => setSelectedTenant(e.target.value)}
                  className="w-full px-3 py-2 bg-ui-bg-subtle border border-ui-border-base rounded-lg text-xs text-ui-fg-base"
                >
                  {tenants.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name} ({t.id})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-ui-fg-subtle mb-1">Session Duration</label>
                <select
                  value={duration}
                  onChange={(e) => setDuration(Number(e.target.value))}
                  className="w-full px-3 py-2 bg-ui-bg-subtle border border-ui-border-base rounded-lg text-xs text-ui-fg-base"
                >
                  <option value={15}>15 Minutes</option>
                  <option value={30}>30 Minutes</option>
                  <option value={60}>60 Minutes (Max)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-ui-fg-subtle mb-1">Audit Reason (Required)</label>
                <input
                  type="text"
                  placeholder="e.g. Troubleshooting broken order checkout state"
                  value={reason}
                  onChange={(e) => setReason(e.target.value)}
                  className="w-full px-3 py-2 bg-ui-bg-subtle border border-ui-border-base rounded-lg text-xs text-ui-fg-base"
                />
              </div>

              <button
                type="submit"
                disabled={loading || !reason.trim()}
                className="w-full py-2 bg-ui-button-inverted hover:bg-ui-button-inverted-hover disabled:opacity-50 text-xs font-semibold text-white rounded-lg transition"
              >
                Launch Support Session
              </button>
            </form>
          )}
        </div>

        {/* Emergency Maintenance & Job Dispatch */}
        <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-ui-fg-base">Incident Controls & Jobs</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/60 text-amber-600 dark:text-amber-400 border border-amber-200 dark:border-amber-800/60">
              High Privilege
            </span>
          </div>

          {/* Read-only toggle */}
          <div className="p-4 bg-ui-bg-subtle border border-ui-border-base rounded-lg space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-ui-fg-base">Lock Tenant Read-Only</span>
              <span
                className={`text-[10px] uppercase font-bold px-2 py-0.5 rounded ${
                  isReadOnly
                    ? "bg-red-50 dark:bg-red-950/60 text-red-700 dark:text-red-400 border border-red-200 dark:border-red-800/60"
                    : "bg-emerald-50 dark:bg-emerald-950/60 text-emerald-600 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/60"
                }`}
              >
                {isReadOnly ? "Read-Only Locked" : "Standard Writable"}
              </span>
            </div>
            <input
              type="text"
              placeholder="Reason for locking/unlocking tenant writes"
              value={readOnlyReason}
              onChange={(e) => setReadOnlyReason(e.target.value)}
              className="w-full px-3 py-1.5 bg-ui-bg-component border border-ui-border-base rounded text-xs text-ui-fg-base"
            />
            <button
              onClick={handleToggleReadOnly}
              disabled={!readOnlyReason.trim()}
              className="px-3 py-1.5 bg-ui-bg-component hover:bg-ui-bg-component-hover disabled:opacity-50 text-xs font-medium text-ui-fg-base rounded transition"
            >
              Toggle Tenant Read-Only
            </button>
          </div>

          {/* Manual Job Triggers */}
          <div className="space-y-2 pt-2">
            <div className="text-xs font-semibold text-ui-fg-subtle">Dispatch System Jobs</div>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
              <button
                onClick={() => handleTriggerJob("reconciliation_sweep")}
                className="p-2.5 bg-ui-bg-subtle border border-ui-border-base hover:border-ui-border-strong rounded-lg text-left text-xs text-ui-fg-subtle transition"
              >
                <div className="font-semibold text-ui-fg-base">Payment Sweep</div>
                <div className="text-[10px] text-ui-fg-muted">Unsettled recovery</div>
              </button>
              <button
                onClick={() => handleTriggerJob("webhook_replay")}
                className="p-2.5 bg-ui-bg-subtle border border-ui-border-base hover:border-ui-border-strong rounded-lg text-left text-xs text-ui-fg-subtle transition"
              >
                <div className="font-semibold text-ui-fg-base">Webhook Replay</div>
                <div className="text-[10px] text-ui-fg-muted">Dead-letter queue</div>
              </button>
              <button
                onClick={() => handleTriggerJob("search_reindex")}
                className="p-2.5 bg-ui-bg-subtle border border-ui-border-base hover:border-ui-border-strong rounded-lg text-left text-xs text-ui-fg-subtle transition"
              >
                <div className="font-semibold text-ui-fg-base">Search Re-index</div>
                <div className="text-[10px] text-ui-fg-muted">Catalog sync</div>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Cross-Tenant Workflow Observability */}
      <div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-ui-fg-base">Workflow Execution Observability</h3>
          <span className="text-xs text-ui-fg-subtle">Live background saga runners</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="text-ui-fg-muted uppercase border-b border-ui-border-base">
              <tr>
                <th className="pb-2">Workflow ID</th>
                <th className="pb-2">Tenant</th>
                <th className="pb-2">Name</th>
                <th className="pb-2">Current Step</th>
                <th className="pb-2">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-ui-border-base text-ui-fg-subtle">
              {workflows.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-4 text-center text-ui-fg-muted italic">
                    All background workflows completed. Zero stranded sagas.
                  </td>
                </tr>
              ) : (
                workflows.map((w) => (
                  <tr key={w.id}>
                    <td className="py-2.5 font-mono text-ui-fg-subtle">{w.id}</td>
                    <td className="py-2.5">{w.tenant_id}</td>
                    <td className="py-2.5 font-medium text-ui-fg-base">{w.workflow_name}</td>
                    <td className="py-2.5">{w.current_step}</td>
                    <td className="py-2.5">
                      <span className="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-emerald-50 dark:bg-emerald-950/60 text-emerald-600 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/60">
                        {w.status}
                      </span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
