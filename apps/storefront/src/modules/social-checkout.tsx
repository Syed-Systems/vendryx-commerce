import React, { useState } from 'react';

export interface SocialCheckoutProps {
  productTitle: string;
  variantTitle?: string;
  priceFormatted: string;
  productUrl: string;
  whatsappNumber: string; // e.g. "1234567890"
  merchantHandle?: string;
}

/**
 * Social Checkout: WhatsApp pre-fill link and 1-click clipboard copy with app launch
 * for Instagram DMs and Facebook Messenger (PRODUCT §5.4).
 */
export const SocialCheckout: React.FC<SocialCheckoutProps> = ({
  productTitle,
  variantTitle,
  priceFormatted,
  productUrl,
  whatsappNumber,
}) => {
  const [copied, setCopied] = useState(false);

  const orderSummaryText = `Hi! I would like to order "${productTitle}${variantTitle ? ` (${variantTitle})` : ''}" for ${priceFormatted}.\nProduct Link: ${productUrl}`;

  const whatsappUrl = `https://wa.me/${whatsappNumber}?text=${encodeURIComponent(orderSummaryText)}`;

  const handleCopyAndOpen = (platformUrl: string) => {
    navigator.clipboard.writeText(orderSummaryText);
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
    window.open(platformUrl, '_blank');
  };

  return (
    <div className="social-checkout p-4 bg-slate-50 border rounded-lg space-y-3">
      <h3 className="text-sm font-semibold text-slate-700">Order via Social Apps:</h3>
      
      <div className="flex flex-col sm:flex-row gap-2">
        {/* WhatsApp Direct */}
        <a
          href={whatsappUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="flex-1 bg-green-600 hover:bg-green-700 text-white text-center py-2 px-3 rounded font-medium text-sm flex items-center justify-center gap-2"
        >
          <span>💬 Order on WhatsApp</span>
        </a>

        {/* Instagram DM (Clipboard + Open) */}
        <button
          type="button"
          onClick={() => handleCopyAndOpen('https://instagram.com/direct/inbox')}
          className="flex-1 bg-gradient-to-r from-purple-500 to-pink-500 hover:opacity-90 text-white py-2 px-3 rounded font-medium text-sm flex items-center justify-center gap-2"
        >
          <span>📸 {copied ? 'Copied & Opening IG!' : 'Order on Instagram'}</span>
        </button>

        {/* Messenger (Clipboard + Open) */}
        <button
          type="button"
          onClick={() => handleCopyAndOpen('https://m.me/')}
          className="flex-1 bg-blue-600 hover:bg-blue-700 text-white py-2 px-3 rounded font-medium text-sm flex items-center justify-center gap-2"
        >
          <span>⚡ Order on Messenger</span>
        </button>
      </div>
    </div>
  );
};
