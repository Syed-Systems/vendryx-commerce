import React, { useEffect, useState } from "react"
import {
  endImpersonation,
  getTenantErrors,
  ImpersonationResponse,
  listTenants,
  listWorkflows,
  startImpersonation,
  Tenant,
  TenantError,
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
  const [errors, setErrors] = useState<TenantError[]>([])
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

  const loadTenantErrors = async () => {
    if (!selectedTenant) return
    try {
      const res = await getTenantErrors(selectedTenant)
      setErrors(res.errors)
    } catch (err: any) {
      setStatusMessage(`Failed to load errors: ${err.message}`)
    }
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold text-zinc-100 tracking-tight">Support & Incident Operations</h2>
        <p className="text-sm text-zinc-400 mt-1">
          Time-boxed merchant impersonation, emergency maintenance locks, and manual system job dispatch.
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

      {/* Grid: Impersonation & Emergency Locks */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Support Impersonation */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-zinc-200">Merchant Impersonation</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-indigo-950/60 text-indigo-400 border border-indigo-800/60">
              ADR-018 Audited
            </span>
          </div>
          <p className="text-xs text-zinc-400">
            Assume merchant scope with server-side time boxing. Secrets and billing modification are strictly barred.
          </p>

          {activeSession ? (
            <div className="p-4 bg-zinc-950 border border-indigo-500/40 rounded-lg space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-indigo-400">Active Support Session</span>
                <span className="text-xs text-zinc-400 font-mono">
                  Expires: {new Date(activeSession.expires_at).toLocaleTimeString()}
                </span>
              </div>
              <div className="p-2 bg-zinc-900 rounded font-mono text-xs text-zinc-300 truncate">
                {activeSession.token}
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => navigator.clipboard.writeText(activeSession.token)}
                  className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-xs font-medium text-zinc-200 rounded transition"
                >
                  Copy Session Token
                </button>
                <button
                  onClick={handleEndImpersonate}
                  className="px-3 py-1.5 bg-red-950/80 hover:bg-red-900 text-xs font-medium text-red-300 rounded border border-red-800 transition"
                >
                  End Session Now
                </button>
              </div>
            </div>
          ) : (
            <form onSubmit={handleStartImpersonate} className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Select Tenant</label>
                <select
                  value={selectedTenant}
                  onChange={(e) => setSelectedTenant(e.target.value)}
                  className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200"
                >
                  {tenants.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name} ({t.id})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Session Duration</label>
                <select
                  value={duration}
                  onChange={(e) => setDuration(Number(e.target.value))}
                  className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200"
                >
                  <option value={15}>15 Minutes</option>
                  <option value={30}>30 Minutes</option>
                  <option value={60}>60 Minutes (Max)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1">Audit Reason (Required)</label>
                <input
                  type="text"
                  placeholder="e.g. Troubleshooting broken order checkout state"
                  value={reason}
                  onChange={(e) => setReason(e.target.value)}
                  className="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200"
                />
              </div>

              <button
                type="submit"
                disabled={loading || !reason.trim()}
                className="w-full py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-xs font-semibold text-white rounded-lg transition"
              >
                Launch Support Session
              </button>
            </form>
          )}
        </div>

        {/* Emergency Maintenance & Job Dispatch */}
        <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-base font-semibold text-zinc-200">Incident Controls & Jobs</h3>
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-amber-950/60 text-amber-400 border border-amber-800/60">
              High Privilege
            </span>
          </div>

          {/* Read-only toggle */}
          <div className="p-4 bg-zinc-950 border border-zinc-800 rounded-lg space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-zinc-200">Lock Tenant Read-Only</span>
              <span
                className={`text-[10px] uppercase font-bold px-2 py-0.5 rounded ${
                  isReadOnly
                    ? "bg-red-950/60 text-red-400 border border-red-800/60"
                    : "bg-emerald-950/60 text-emerald-400 border border-emerald-800/60"
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
              className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded text-xs text-zinc-200"
            />
            <button
              onClick={handleToggleReadOnly}
              disabled={!readOnlyReason.trim()}
              className="px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-xs font-medium text-zinc-200 rounded transition"
            >
              Toggle Tenant Read-Only
            </button>
          </div>

          {/* Manual Job Triggers */}
          <div className="space-y-2 pt-2">
            <div className="text-xs font-semibold text-zinc-300">Dispatch System Jobs</div>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
              <button
                onClick={() => handleTriggerJob("reconciliation_sweep")}
                className="p-2.5 bg-zinc-950 border border-zinc-800 hover:border-zinc-700 rounded-lg text-left text-xs text-zinc-300 transition"
              >
                <div className="font-semibold text-zinc-200">Payment Sweep</div>
                <div className="text-[10px] text-zinc-500">Unsettled recovery</div>
              </button>
              <button
                onClick={() => handleTriggerJob("webhook_replay")}
                className="p-2.5 bg-zinc-950 border border-zinc-800 hover:border-zinc-700 rounded-lg text-left text-xs text-zinc-300 transition"
              >
                <div className="font-semibold text-zinc-200">Webhook Replay</div>
                <div className="text-[10px] text-zinc-500">Dead-letter queue</div>
              </button>
              <button
                onClick={() => handleTriggerJob("search_reindex")}
                className="p-2.5 bg-zinc-950 border border-zinc-800 hover:border-zinc-700 rounded-lg text-left text-xs text-zinc-300 transition"
              >
                <div className="font-semibold text-zinc-200">Search Re-index</div>
                <div className="text-[10px] text-zinc-500">Catalog sync</div>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Cross-Tenant Workflow Observability */}
      <div className="p-6 bg-zinc-900/50 border border-zinc-800 rounded-xl space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-zinc-200">Workflow Execution Observability</h3>
          <span className="text-xs text-zinc-400">Live background saga runners</span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="text-zinc-500 uppercase border-b border-zinc-800">
              <tr>
                <th className="pb-2">Workflow ID</th>
                <th className="pb-2">Tenant</th>
                <th className="pb-2">Name</th>
                <th className="pb-2">Current Step</th>
                <th className="pb-2">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-800/60 text-zinc-300">
              {workflows.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-4 text-center text-zinc-500 italic">
                    All background workflows completed. Zero stranded sagas.
                  </td>
                </tr>
              ) : (
                workflows.map((w) => (
                  <tr key={w.id}>
                    <td className="py-2.5 font-mono text-zinc-400">{w.id}</td>
                    <td className="py-2.5">{w.tenant_id}</td>
                    <td className="py-2.5 font-medium text-zinc-200">{w.workflow_name}</td>
                    <td className="py-2.5">{w.current_step}</td>
                    <td className="py-2.5">
                      <span className="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-emerald-950/60 text-emerald-400 border border-emerald-800/60">
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
