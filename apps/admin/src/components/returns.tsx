import React, { useState } from 'react';

export interface ReturnRecord {
  id: string;
  order_id: string;
  customer_name: string;
  status: 'requested' | 'received' | 'refunded' | 'canceled';
  reason: string;
  refund_amount: string;
}

export const ReturnsManagement: React.FC = () => {
  const [returns] = useState<ReturnRecord[]>([
    { id: 'ret_01', order_id: 'ord_101', customer_name: 'Jane Doe', status: 'received', reason: 'Incorrect size (too small)', refund_amount: '$25.00' },
    { id: 'ret_02', order_id: 'ord_102', customer_name: 'John Smith', status: 'requested', reason: 'Defective zipper', refund_amount: '$140.00' },
  ]);

  return (
    <div className="returns-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Returns, Claims & Exchanges</h2>
          <p className="text-sm text-slate-500">Process return shipments, examine return reasons, and issue item exchanges.</p>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Return ID</th>
              <th className="p-3">Order</th>
              <th className="p-3">Customer</th>
              <th className="p-3">Reason</th>
              <th className="p-3">Refund Amount</th>
              <th className="p-3">Status</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {returns.map((r) => (
              <tr key={r.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-mono text-xs font-semibold text-slate-900">{r.id}</td>
                <td className="p-3 text-slate-600">{r.order_id}</td>
                <td className="p-3 text-slate-800">{r.customer_name}</td>
                <td className="p-3 text-xs text-slate-600">{r.reason}</td>
                <td className="p-3 font-medium text-slate-900">{r.refund_amount}</td>
                <td className="p-3">
                  <span className="text-xs px-2 py-0.5 rounded font-medium bg-amber-50 text-amber-700">
                    {r.status}
                  </span>
                </td>
                <td className="p-3 text-right">
                  <button className="text-emerald-600 hover:text-emerald-800 text-xs font-medium mr-2">
                    Receive
                  </button>
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium">
                    Issue Refund
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
