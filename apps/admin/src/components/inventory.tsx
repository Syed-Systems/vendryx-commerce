import React, { useState } from 'react';

export interface InventoryLevel {
  id: string;
  sku: string;
  title: string;
  location_name: string;
  stocked_quantity: number;
  reserved_quantity: number;
  available_quantity: number;
}

export const InventoryManager: React.FC = () => {
  const [levels] = useState<InventoryLevel[]>([
    { id: 'inv_1', sku: 'SHIRT-S-BLK', title: 'Cotton Crewneck (S)', location_name: 'Primary Warehouse', stocked_quantity: 50, reserved_quantity: 5, available_quantity: 45 },
    { id: 'inv_2', sku: 'SHIRT-M-BLK', title: 'Cotton Crewneck (M)', location_name: 'Primary Warehouse', stocked_quantity: 90, reserved_quantity: 10, available_quantity: 80 },
    { id: 'inv_3', sku: 'HOOD-L-GRY', title: 'Fleece Hoodie (L)', location_name: 'Secondary Hub', stocked_quantity: 35, reserved_quantity: 0, available_quantity: 35 },
  ]);

  return (
    <div className="inventory-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Inventory & Stock Locations</h2>
          <p className="text-sm text-slate-500">Monitor multi-location stock levels and item reservations.</p>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">SKU / Item</th>
              <th className="p-3">Location</th>
              <th className="p-3">Stocked</th>
              <th className="p-3">Reserved</th>
              <th className="p-3">Available</th>
            </tr>
          </thead>
          <tbody>
            {levels.map((lvl) => (
              <tr key={lvl.id} className="border-b hover:bg-slate-50">
                <td className="p-3">
                  <div className="font-medium text-slate-900">{lvl.title}</div>
                  <div className="text-xs font-mono text-slate-500">{lvl.sku}</div>
                </td>
                <td className="p-3 text-slate-600">{lvl.location_name}</td>
                <td className="p-3 text-slate-800">{lvl.stocked_quantity}</td>
                <td className="p-3 text-amber-600">{lvl.reserved_quantity}</td>
                <td className="p-3 font-semibold text-emerald-700">{lvl.available_quantity}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
