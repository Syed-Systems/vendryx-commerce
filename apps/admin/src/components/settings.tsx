import React, { useState } from 'react';

export const StoreSettings: React.FC = () => {
  const [storeName, setStoreName] = useState('Stheno Flagship Store');
  const [defaultCurrency, setDefaultCurrency] = useState('usd');
  const [regions] = useState([
    { id: 'reg_na', name: 'North America', currency: 'USD', countries: ['US', 'CA'] },
    { id: 'reg_eu', name: 'European Union', currency: 'EUR', countries: ['DE', 'FR', 'ES', 'IT'] },
  ]);

  return (
    <div className="settings-container p-6 bg-white rounded-lg shadow-sm border space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Store Settings & Configuration (P18-19)</h2>
        <p className="text-sm text-slate-500">Configure regions, currencies, sales channels, and tax rates.</p>
      </div>

      <div className="space-y-4 max-w-xl">
        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Store Name
          </label>
          <input
            type="text"
            value={storeName}
            onChange={(e) => setStoreName(e.target.value)}
            className="w-full px-3 py-2 border rounded text-sm focus:outline-none focus:ring-1 focus:ring-slate-900"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Default Base Currency
          </label>
          <select
            value={defaultCurrency}
            onChange={(e) => setDefaultCurrency(e.target.value)}
            className="w-full px-3 py-2 border rounded text-sm bg-white focus:outline-none focus:ring-1 focus:ring-slate-900"
          >
            <option value="usd">USD - US Dollar ($)</option>
            <option value="eur">EUR - Euro (€)</option>
            <option value="gbp">GBP - British Pound (£)</option>
          </select>
        </div>
      </div>

      <div className="border-t pt-4 space-y-3">
        <h3 className="font-semibold text-sm text-slate-800">Configured Regions</h3>
        <div className="grid gap-3">
          {regions.map((r) => (
            <div key={r.id} className="p-3 border rounded bg-slate-50 flex justify-between items-center">
              <div>
                <p className="font-medium text-slate-900 text-sm">{r.name}</p>
                <p className="text-xs text-slate-500">Currency: {r.currency} • Countries: {r.countries.join(', ')}</p>
              </div>
              <button className="text-xs text-indigo-600 hover:text-indigo-800 font-medium">
                Manage Region
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
