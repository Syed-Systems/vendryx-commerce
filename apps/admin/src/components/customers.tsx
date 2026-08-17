import React, { useState } from 'react';

export interface CustomerRow {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  orders_count: number;
  total_spent: string;
}

export const CustomersView: React.FC = () => {
  const [customers] = useState<CustomerRow[]>([
    { id: 'cus_01', email: 'jane@example.com', first_name: 'Jane', last_name: 'Doe', orders_count: 5, total_spent: '$420.00' },
    { id: 'cus_02', email: 'john@example.com', first_name: 'John', last_name: 'Smith', orders_count: 2, total_spent: '$185.00' },
    { id: 'cus_03', email: 'alex@example.com', first_name: 'Alex', last_name: 'Rivera', orders_count: 1, total_spent: '$45.00' },
  ]);

  return (
    <div className="customers-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Customers & Groups (P18-17)</h2>
          <p className="text-sm text-slate-500">Manage buyer accounts, lifetime order metrics, and VIP segmentations.</p>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Customer</th>
              <th className="p-3">Email</th>
              <th className="p-3">Orders</th>
              <th className="p-3">Total Spend</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {customers.map((c) => (
              <tr key={c.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-medium text-slate-900">{c.first_name} {c.last_name}</td>
                <td className="p-3 text-slate-600 font-mono text-xs">{c.email}</td>
                <td className="p-3 text-slate-700">{c.orders_count}</td>
                <td className="p-3 font-medium text-slate-900">{c.total_spent}</td>
                <td className="p-3 text-right">
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium">
                    View Account
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
