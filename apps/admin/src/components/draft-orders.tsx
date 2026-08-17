import React, { useState } from 'react';

export interface DraftOrderRow {
  id: string;
  display_id: number;
  customer_email: string;
  status: 'open' | 'completed';
  total: string;
  created_at: string;
}

export const DraftOrdersView: React.FC = () => {
  const [drafts] = useState<DraftOrderRow[]>([
    { id: 'dord_01', display_id: 501, customer_email: 'wholesale@boutique.com', status: 'open', total: '$1,250.00', created_at: '2026-08-17' },
    { id: 'dord_02', display_id: 502, customer_email: 'custom@client.org', status: 'completed', total: '$320.00', created_at: '2026-08-16' },
  ]);

  return (
    <div className="draft-orders-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Draft Orders</h2>
          <p className="text-sm text-slate-500">Create manual quotes, wholesale orders, and phone invoices.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + Create Draft Order
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Draft #</th>
              <th className="p-3">Customer Email</th>
              <th className="p-3">Total Amount</th>
              <th className="p-3">Status</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {drafts.map((d) => (
              <tr key={d.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-semibold text-slate-900">#{d.display_id}</td>
                <td className="p-3 text-slate-700">{d.customer_email}</td>
                <td className="p-3 font-medium text-slate-900">{d.total}</td>
                <td className="p-3">
                  <span className="text-xs px-2 py-0.5 rounded font-medium bg-blue-50 text-blue-700">
                    {d.status}
                  </span>
                </td>
                <td className="p-3 text-right">
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium mr-2">
                    Send Invoice
                  </button>
                  <button className="text-emerald-600 hover:text-emerald-800 text-xs font-medium">
                    Convert to Order
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
