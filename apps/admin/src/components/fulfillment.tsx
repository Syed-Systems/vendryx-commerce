import React, { useState } from 'react';

export interface ShippingOption {
  id: string;
  name: string;
  price_type: 'flat' | 'calculated';
  amount: string;
  provider_id: string;
}

export const FulfillmentManagement: React.FC = () => {
  const [options] = useState<ShippingOption[]>([
    { id: 'so_std', name: 'Standard Ground Shipping (3-5 business days)', price_type: 'flat', amount: '$5.00', provider_id: 'manual' },
    { id: 'so_exp', name: 'Express Overnight Air', price_type: 'flat', amount: '$20.00', provider_id: 'manual' },
  ]);

  return (
    <div className="fulfillment-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Shipping & Fulfillment (P18-16)</h2>
          <p className="text-sm text-slate-500">Configure shipping options, fulfillment sets, and carrier providers.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + Add Shipping Option
        </button>
      </div>

      <div className="grid gap-4">
        {options.map((opt) => (
          <div key={opt.id} className="border p-4 rounded-lg flex justify-between items-center bg-slate-50">
            <div>
              <p className="font-semibold text-slate-900 text-sm">{opt.name}</p>
              <p className="text-xs text-slate-500 mt-0.5">Provider: {opt.provider_id} • Rate: {opt.amount}</p>
            </div>
            <button className="text-xs text-indigo-600 hover:text-indigo-800 font-medium">
              Configure Option
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
