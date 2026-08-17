import React, { useState } from 'react';

export interface PromotionRow {
  id: string;
  code: string;
  type: 'standard' | 'buy_get';
  value: string;
  status: 'active' | 'scheduled' | 'expired';
}

export const MarketingPromotions: React.FC = () => {
  const [promotions] = useState<PromotionRow[]>([
    { id: 'promo_1', code: 'SUMMER20', type: 'standard', value: '20% OFF Entire Order', status: 'active' },
    { id: 'promo_2', code: 'FREESHIP', type: 'standard', value: 'Free Ground Shipping', status: 'active' },
    { id: 'promo_3', code: 'BOGO_TEE', type: 'buy_get', value: 'Buy 1 Get 1 50% Off', status: 'scheduled' },
  ]);

  return (
    <div className="marketing-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Promotions & Campaigns (P18-18)</h2>
          <p className="text-sm text-slate-500">Configure discount rules, promotional codes, and campaign timelines.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + New Promotion
        </button>
      </div>

      <div className="grid gap-4">
        {promotions.map((p) => (
          <div key={p.id} className="border p-4 rounded-lg flex justify-between items-center bg-slate-50">
            <div>
              <div className="flex items-center gap-2">
                <span className="font-mono font-bold text-slate-900 text-sm bg-white px-2 py-0.5 border rounded">
                  {p.code}
                </span>
                <span
                  className={`text-xs px-2 py-0.5 rounded font-medium ${
                    p.status === 'active' ? 'bg-emerald-50 text-emerald-700' : 'bg-blue-50 text-blue-700'
                  }`}
                >
                  {p.status}
                </span>
              </div>
              <p className="text-xs text-slate-600 mt-1 font-medium">{p.value}</p>
            </div>
            <button className="text-xs text-indigo-600 hover:text-indigo-800 font-medium">
              Edit Campaign
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
