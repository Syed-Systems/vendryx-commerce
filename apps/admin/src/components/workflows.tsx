import React from 'react';

export interface WorkflowExecutionStep {
  name: string;
  status: 'completed' | 'running' | 'compensated' | 'failed';
  durationMs: number;
}

export const WorkflowObservability: React.FC = () => {
  const steps: WorkflowExecutionStep[] = [
    { name: 'validate-cart', status: 'completed', durationMs: 12 },
    { name: 'reserve-inventory', status: 'completed', durationMs: 18 },
    { name: 'authorize-payment', status: 'completed', durationMs: 84 },
    { name: 'create-order', status: 'completed', durationMs: 25 },
    { name: 'emit-order-placed-event', status: 'completed', durationMs: 6 },
  ];

  return (
    <div className="workflows-card p-6 bg-white border rounded-xl shadow-sm space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Workflow Execution Timeline</h2>
          <p className="text-sm text-slate-500">Transaction ID: `wf_exec_01G8Z99` (complete-cart)</p>
        </div>
        <span className="bg-emerald-100 text-emerald-800 text-xs px-3 py-1 rounded font-semibold">
          SUCCESS (145 ms total)
        </span>
      </div>

      <div className="border rounded-lg divide-y">
        {steps.map((step, idx) => (
          <div key={idx} className="p-3 flex items-center justify-between text-sm">
            <div className="flex items-center gap-3">
              <span className="w-6 h-6 rounded-full bg-emerald-500 text-white text-xs flex items-center justify-center font-bold">
                ✓
              </span>
              <span className="font-mono font-medium text-slate-800">{step.name}</span>
            </div>
            <span className="text-slate-400 font-mono text-xs">{step.durationMs} ms</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export const WorkflowMonitor = WorkflowObservability;
