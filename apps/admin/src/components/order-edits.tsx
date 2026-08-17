import React, { useState } from 'react';

export const OrderEditPreview: React.FC = () => {
  const [items, setItems] = useState([
    { id: 'item_1', title: 'Cotton Crewneck T-Shirt (M)', quantity: 2, unit_price: 25.0 },
    { id: 'item_2', title: 'Canvas Tote Bag', quantity: 1, unit_price: 15.0 },
  ]);

  const subtotal = items.reduce((sum, item) => sum + item.quantity * item.unit_price, 0);
  const tax = subtotal * 0.1;
  const total = subtotal + tax;

  return (
    <div className="order-edits-container p-6 bg-white rounded-lg shadow-sm border space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Order Edit & Live Preview (P18-13)</h2>
        <p className="text-sm text-slate-500">Add, adjust, or remove line items with automatic refund/difference calculation.</p>
      </div>

      <div className="border rounded-lg p-4 space-y-3 bg-slate-50">
        <h3 className="font-semibold text-sm text-slate-800">Adjust Items</h3>
        {items.map((item, idx) => (
          <div key={item.id} className="flex justify-between items-center bg-white p-3 rounded border">
            <div>
              <p className="text-sm font-medium text-slate-900">{item.title}</p>
              <p className="text-xs text-slate-500">${item.unit_price.toFixed(2)} each</p>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-sm text-slate-700">Qty: {item.quantity}</span>
              <span className="text-sm font-semibold text-slate-900">
                ${(item.quantity * item.unit_price).toFixed(2)}
              </span>
            </div>
          </div>
        ))}
      </div>

      <div className="border-t pt-4 space-y-1 text-sm max-w-xs ml-auto">
        <div className="flex justify-between text-slate-600">
          <span>Subtotal</span>
          <span>${subtotal.toFixed(2)}</span>
        </div>
        <div className="flex justify-between text-slate-600">
          <span>Estimated Tax (10%)</span>
          <span>${tax.toFixed(2)}</span>
        </div>
        <div className="flex justify-between font-bold text-slate-900 text-base pt-2 border-t">
          <span>New Order Total</span>
          <span>${total.toFixed(2)}</span>
        </div>
      </div>

      <div className="flex justify-end gap-3">
        <button className="px-4 py-2 border rounded text-sm text-slate-700 hover:bg-slate-50 font-medium">
          Cancel Edit
        </button>
        <button className="px-4 py-2 bg-slate-900 text-white rounded text-sm hover:bg-slate-800 font-medium">
          Confirm & Request Payment Difference
        </button>
      </div>
    </div>
  );
};
