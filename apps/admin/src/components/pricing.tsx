import React, { useState } from 'react';

export interface PriceListSummary {
  id: string;
  title: string;
  description: string;
  status: 'active' | 'draft';
  rules_count: number;
}

export const PricingManager: React.FC = () => {
  const [priceLists] = useState<PriceListSummary[]>([
    { id: 'plist_vip', title: 'VIP Customer Discount (20% Off)', description: 'Applied automatically to VIP customer group', status: 'active', rules_count: 12 },
    { id: 'plist_summer', title: 'Summer Flash Sale', description: 'Selected seasonal apparel lines', status: 'draft', rules_count: 8 },
  ]);

  return (
    <div className="pricing-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Price Lists & Overrides</h2>
          <p className="text-sm text-slate-500">Manage customer-group pricing, sale rules, and currency preferences.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + New Price List
        </button>
      </div>

      <div className="grid gap-4">
        {priceLists.map((pl) => (
          <div key={pl.id} className="border p-4 rounded-lg flex justify-between items-center">
            <div>
              <div className="flex items-center gap-2">
                <span className="font-semibold text-slate-900 text-sm">{pl.title}</span>
                <span
                  className={`text-xs px-2 py-0.5 rounded font-medium ${
                    pl.status === 'active' ? 'bg-emerald-50 text-emerald-700' : 'bg-slate-100 text-slate-600'
                  }`}
                >
                  {pl.status}
                </span>
              </div>
              <p className="text-xs text-slate-500 mt-1">{pl.description}</p>
              <p className="text-xs text-slate-600 mt-2 font-medium">{pl.rules_count} price override rules</p>
            </div>
            <button className="text-xs text-indigo-600 hover:text-indigo-800 font-medium">
              Configure Rules
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
