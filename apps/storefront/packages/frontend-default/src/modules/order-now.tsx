import React, { useState } from 'react';

export interface OrderNowProps {
  productId: string;
  variantId: string;
  unitPriceMinor: number;
  currencyCode: string;
  onOrderCompleted?: (orderId: string) => void;
}

/**
 * Order Now component: Single-page fast checkout with address form and Cash on Delivery (COD) default.
 */
export const OrderNow: React.FC<OrderNowProps> = ({
  productId,
  variantId,
  unitPriceMinor,
  currencyCode,
  onOrderCompleted,
}) => {
  const [shippingAddress, setShippingAddress] = useState({
    firstName: '',
    lastName: '',
    addressLine: '',
    city: '',
    phone: '',
  });
  const [paymentMethod, setPaymentMethod] = useState<'cod' | 'card'>('cod');
  const [loading, setLoading] = useState(false);
  const [orderId, setOrderId] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    // Call Stheno fast-checkout / complete-cart endpoint
    const response = await fetch('/api/store/order-now', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        variant_id: variantId,
        quantity: 1,
        shipping_address: shippingAddress,
        payment_method: paymentMethod,
      }),
    });

    const data = await response.json();
    setLoading(false);
    if (data.order?.id) {
      setOrderId(data.order.id);
      onOrderCompleted?.(data.order.id);
    }
  };

  return (
    <div className="order-now-container p-6 bg-[color:var(--brand-surface)] rounded-[var(--brand-radius-card)] border">
      <h2 className="text-xl font-bold mb-4">Fast Order / Cash on Delivery</h2>
      {orderId ? (
        <div className="text-green-600 font-semibold">
          Order placed successfully! Order ID: {orderId}
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <input
              type="text"
              placeholder="First Name"
              required
              value={shippingAddress.firstName}
              onChange={(e) => setShippingAddress({ ...shippingAddress, firstName: e.target.value })}
              className="border p-2 rounded"
            />
            <input
              type="text"
              placeholder="Last Name"
              required
              value={shippingAddress.lastName}
              onChange={(e) => setShippingAddress({ ...shippingAddress, lastName: e.target.value })}
              className="border p-2 rounded"
            />
          </div>
          <input
            type="text"
            placeholder="Delivery Address"
            required
            value={shippingAddress.addressLine}
            onChange={(e) => setShippingAddress({ ...shippingAddress, addressLine: e.target.value })}
            className="w-full border p-2 rounded"
          />
          <div className="grid grid-cols-2 gap-4">
            <input
              type="text"
              placeholder="City"
              required
              value={shippingAddress.city}
              onChange={(e) => setShippingAddress({ ...shippingAddress, city: e.target.value })}
              className="border p-2 rounded"
            />
            <input
              type="tel"
              placeholder="Phone Number"
              required
              value={shippingAddress.phone}
              onChange={(e) => setShippingAddress({ ...shippingAddress, phone: e.target.value })}
              className="border p-2 rounded"
            />
          </div>

          <div className="payment-options">
            <label className="flex items-center space-x-2">
              <input
                type="radio"
                name="payment"
                value="cod"
                checked={paymentMethod === 'cod'}
                onChange={() => setPaymentMethod('cod')}
              />
              <span>Cash on Delivery (Pay when you receive)</span>
            </label>
          </div>

          <button
            type="submit"
            disabled={loading}
            className="w-full bg-emerald-600 text-[color:var(--on-dark)] font-bold py-3 rounded hover:bg-emerald-700"
          >
            {loading ? 'Placing Order...' : `Order Now • ${(unitPriceMinor / 100).toFixed(2)} ${currencyCode.toUpperCase()}`}
          </button>
        </form>
      )}
    </div>
  );
};
