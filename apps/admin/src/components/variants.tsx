import React, { useState } from 'react';

export interface VariantItem {
  id: string;
  title: string;
  sku: string;
  inventory_quantity: number;
  options: Record<string, string>;
}

export const VariantsManager: React.FC = () => {
  const [variants] = useState<VariantItem[]>([
    { id: 'var_1', title: 'Small / Black', sku: 'SHIRT-S-BLK', inventory_quantity: 45, options: { Size: 'S', Color: 'Black' } },
    { id: 'var_2', title: 'Medium / Black', sku: 'SHIRT-M-BLK', inventory_quantity: 80, options: { Size: 'M', Color: 'Black' } },
    { id: 'var_3', title: 'Large / Black', sku: 'SHIRT-L-BLK', inventory_quantity: 30, options: { Size: 'L', Color: 'Black' } },
  ]);

  return (
    <div className="variants-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Product Variants & Options</h2>
          <p className="text-sm text-slate-500">Configure size, color, material variations and linked SKUs.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + Add Variant
        </button>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Variant Title</th>
              <th className="p-3">SKU</th>
              <th className="p-3">Options</th>
              <th className="p-3">Available Stock</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {variants.map((v) => (
              <tr key={v.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-medium text-slate-900">{v.title}</td>
                <td className="p-3 font-mono text-xs text-slate-600">{v.sku}</td>
                <td className="p-3">
                  <span className="bg-slate-100 text-slate-700 text-xs px-2 py-0.5 rounded">
                    {Object.entries(v.options).map(([k, val]) => `${k}: ${val}`).join(', ')}
                  </span>
                </td>
                <td className="p-3 text-slate-800">{v.inventory_quantity} units</td>
                <td className="p-3 text-right">
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium">
                    Edit Stock
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
