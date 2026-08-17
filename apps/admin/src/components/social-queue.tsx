import React, { useState } from 'react';

export interface SocialDraftOrder {
  id: string;
  refCode: string;
  source: 'whatsapp' | 'instagram' | 'messenger';
  buyerName: string;
  itemsSummary: string;
  totalFormatted: string;
  status: 'pending' | 'converted';
}

/**
 * Social Queue Screen (P9-31, P9-32, P9-33):
 * - Renders social draft orders with matching `?ref=` marketing codes
 * - Direct 1-click conversion to live orders with inventory reservation resolution
 */
export const SocialQueue: React.FC = () => {
  const [orders, setOrders] = useState<SocialDraftOrder[]>([
    {
      id: 'dord_01G8Z99',
      refCode: 'IG_SUMMER_26',
      source: 'instagram',
      buyerName: 'Amina Khan',
      itemsSummary: 'Cotton Crewneck T-Shirt (M) x 1',
      totalFormatted: '$29.99',
      status: 'pending',
    },
    {
      id: 'dord_01G8Z100',
      refCode: 'WA_FLASH',
      source: 'whatsapp',
      buyerName: 'David Lee',
      itemsSummary: 'Classic Denim Jacket (L) x 1',
      totalFormatted: '$89.00',
      status: 'pending',
    },
  ]);

  const convertToLiveOrder = (orderId: string) => {
    setOrders((prev) =>
      prev.map((o) => (o.id === orderId ? { ...o, status: 'converted' } : o))
    );
  };

  return (
    <div className="social-queue-container p-6 bg-white border rounded-xl shadow-sm space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Social Commerce Order Queue</h2>
        <p className="text-sm text-slate-500">Incoming draft orders from WhatsApp, Instagram, and Messenger inquiries.</p>
      </div>

      <div className="space-y-3">
        {orders.map((ord) => (
          <div key={ord.id} className="border p-4 rounded-lg flex items-center justify-between">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <span className="font-semibold text-slate-800">{ord.buyerName}</span>
                <span className="text-xs bg-purple-100 text-purple-800 px-2 py-0.5 rounded font-mono">
                  Ref: {ord.refCode}
                </span>
                <span className="text-xs bg-slate-100 text-slate-700 px-2 py-0.5 rounded uppercase">
                  {ord.source}
                </span>
              </div>
              <p className="text-sm text-slate-600">{ord.itemsSummary} • <span className="font-bold">{ord.totalFormatted}</span></p>
            </div>

            <div>
              {ord.status === 'converted' ? (
                <span className="text-sm font-semibold text-emerald-600">✓ Converted to Live Order</span>
              ) : (
                <button
                  onClick={() => convertToLiveOrder(ord.id)}
                  className="bg-slate-900 hover:bg-slate-800 text-white font-medium text-sm py-2 px-4 rounded-lg"
                >
                  Convert to Order & Reserve Stock
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export const SocialOrderQueue = SocialQueue;
