import React, { useState } from 'react';

export interface OrderRow {
  id: string;
  display_id: number;
  customer_name: string;
  total: string;
  status: 'pending' | 'completed' | 'canceled' | 'requires_action';
  fulfillment_status: 'not_fulfilled' | 'fulfilled' | 'partially_fulfilled' | 'shipped';
  created_at: string;
}

export const OrdersView: React.FC = () => {
  const [orders] = useState<OrderRow[]>([
    { id: 'ord_101', display_id: 1001, customer_name: 'Jane Doe', total: '$85.00', status: 'completed', fulfillment_status: 'shipped', created_at: '2026-08-16' },
    { id: 'ord_102', display_id: 1002, customer_name: 'John Smith', total: '$140.00', status: 'pending', fulfillment_status: 'not_fulfilled', created_at: '2026-08-17' },
    { id: 'ord_103', display_id: 1003, customer_name: 'Alex Rivera', total: '$45.00', status: 'requires_action', fulfillment_status: 'partially_fulfilled', created_at: '2026-08-17' },
  ]);

  return (
    <div className="orders-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Orders Management</h2>
          <p className="text-sm text-slate-500">Track purchase journey, lifecycle states, payments, and fulfillments.</p>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Order #</th>
              <th className="p-3">Customer</th>
              <th className="p-3">Total</th>
              <th className="p-3">Status</th>
              <th className="p-3">Fulfillment</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {orders.map((ord) => (
              <tr key={ord.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-semibold text-slate-900">#{ord.display_id}</td>
                <td className="p-3 text-slate-700">{ord.customer_name}</td>
                <td className="p-3 font-medium text-slate-900">{ord.total}</td>
                <td className="p-3">
                  <span
                    className={`text-xs px-2 py-0.5 rounded font-medium ${
                      ord.status === 'completed'
                        ? 'bg-emerald-50 text-emerald-700'
                        : ord.status === 'pending'
                        ? 'bg-blue-50 text-blue-700'
                        : 'bg-amber-50 text-amber-700'
                    }`}
                  >
                    {ord.status}
                  </span>
                </td>
                <td className="p-3 text-xs text-slate-600 font-medium">{ord.fulfillment_status}</td>
                <td className="p-3 text-right">
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium mr-2">
                    View Details
                  </button>
                  <button className="text-slate-600 hover:text-slate-800 text-xs font-medium">
                    Edit Order
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
