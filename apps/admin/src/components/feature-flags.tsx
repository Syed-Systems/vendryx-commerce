import React, { useState } from 'react';

export interface FeatureFlag {
  key: string;
  name: string;
  enabled: boolean;
  tier_required: 'free' | 'pro' | 'enterprise';
}

export const FeatureFlagsOverview: React.FC = () => {
  const [flags] = useState<FeatureFlag[]>([
    { key: 'promotions', name: 'Promotions & Discount Engine', enabled: true, tier_required: 'free' },
    { key: 'analytics_advanced', name: 'Extended Analytics Horizon (>30d)', enabled: true, tier_required: 'pro' },
    { key: 'custom_rbac', name: 'Custom RBAC & Roles Matrix', enabled: true, tier_required: 'pro' },
    { key: 'sso_oidc', name: 'Enterprise Single Sign-On (SSO)', enabled: false, tier_required: 'enterprise' },
  ]);

  return (
    <div className="feature-flags-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Feature Flags & Capability Gating (P18-23)</h2>
        <p className="text-sm text-slate-500">
          Features are conditionally mounted by backend router tiers. Unavailable screens are hidden automatically.
        </p>
      </div>

      <div className="grid gap-3">
        {flags.map((flag) => (
          <div key={flag.key} className="p-3 border rounded bg-slate-50 flex justify-between items-center">
            <div>
              <div className="flex items-center gap-2">
                <span className="font-semibold text-slate-900 text-sm">{flag.name}</span>
                <span className="text-xs uppercase bg-slate-200 text-slate-700 px-1.5 py-0.5 rounded font-mono">
                  {flag.tier_required}
                </span>
              </div>
              <p className="text-xs font-mono text-slate-500 mt-0.5">{flag.key}</p>
            </div>
            <span
              className={`text-xs px-2 py-0.5 rounded font-medium ${
                flag.enabled ? 'bg-emerald-50 text-emerald-700' : 'bg-slate-200 text-slate-600'
              }`}
            >
              {flag.enabled ? 'Enabled' : 'Disabled (Tier Gate)'}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};
