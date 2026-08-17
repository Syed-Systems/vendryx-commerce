import React from 'react';

/**
 * Billing & Entitlements Screen (P9-35, P9-14f):
 * Shows current plan tier, trial countdown, quota usage, and over-quota grace banner.
 */
export const BillingSettings: React.FC = () => {
  const currentProducts = 85;
  const productQuota = 100;
  const daysLeftInTrial = 14;

  return (
    <div className="billing-container p-6 bg-white border rounded-xl shadow-sm space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Plan, Billing & Quota Usage</h2>
        <p className="text-sm text-slate-500">Manage your subscription tier and view live platform quota limits.</p>
      </div>

      {/* Trial Countdown Banner */}
      <div className="p-4 bg-amber-50 border border-amber-200 rounded-lg flex items-center justify-between">
        <div className="space-y-1">
          <p className="text-sm font-semibold text-amber-900">Pro Plan Trial Active ({daysLeftInTrial} days remaining)</p>
          <p className="text-xs text-amber-700">Enjoy unmetered analytics, custom roles, and Stripe integration during your trial.</p>
        </div>
        <button className="bg-amber-600 hover:bg-amber-700 text-white font-medium text-xs px-3.5 py-2 rounded">
          Upgrade to Pro
        </button>
      </div>

      {/* Quota Progress */}
      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="font-medium text-slate-700">Active Products Quota</span>
          <span className="text-slate-500">{currentProducts} / {productQuota} products used</span>
        </div>
        <div className="w-full bg-slate-100 rounded-full h-2.5">
          <div
            className="bg-emerald-600 h-2.5 rounded-full"
            style={{ width: `${(currentProducts / productQuota) * 100}%` }}
          />
        </div>
        <p className="text-xs text-slate-400">Grandfathering rule active: Downgrading preserves existing products in catalog.</p>
      </div>
    </div>
  );
};
