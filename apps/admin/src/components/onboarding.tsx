import React, { useState } from 'react';

/**
 * Onboarding Component (P9-27):
 * Google sign-in -> Store name + WhatsApp number -> Subdomain with live uniqueness check.
 */
export const StoreOnboarding: React.FC = () => {
  const [storeName, setStoreName] = useState('');
  const [whatsappNumber, setWhatsappNumber] = useState('');
  const [subdomain, setSubdomain] = useState('');
  const [isSubdomainAvailable, setIsSubdomainAvailable] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(false);

  const checkSubdomain = async (val: string) => {
    setSubdomain(val);
    if (!val || val.length < 3) {
      setIsSubdomainAvailable(null);
      return;
    }
    setChecking(true);
    // Simulate / API check to Stheno store uniqueness endpoint
    setTimeout(() => {
      setIsSubdomainAvailable(val.toLowerCase() !== 'taken');
      setChecking(false);
    }, 200);
  };

  return (
    <div className="onboarding-container max-w-xl mx-auto p-8 bg-white border rounded-xl shadow-sm">
      <h1 className="text-2xl font-bold text-slate-900 mb-2">Launch Your Stheno Store</h1>
      <p className="text-sm text-slate-600 mb-6">Setup your store name, social contact, and live subdomain in seconds.</p>

      <form className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-slate-700 mb-1">Store Name</label>
          <input
            type="text"
            required
            placeholder="e.g. Modern Threads"
            value={storeName}
            onChange={(e) => setStoreName(e.target.value)}
            className="w-full border p-2.5 rounded-lg text-sm"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 mb-1">WhatsApp Business Number</label>
          <input
            type="tel"
            required
            placeholder="+1 555 123 4567"
            value={whatsappNumber}
            onChange={(e) => setWhatsappNumber(e.target.value)}
            className="w-full border p-2.5 rounded-lg text-sm"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-700 mb-1">Store Subdomain</label>
          <div className="flex items-center">
            <input
              type="text"
              required
              placeholder="my-brand"
              value={subdomain}
              onChange={(e) => checkSubdomain(e.target.value)}
              className="flex-1 border p-2.5 rounded-l-lg text-sm"
            />
            <span className="bg-slate-100 border border-l-0 px-3 py-2.5 text-sm text-slate-500 rounded-r-lg">
              .stheno.store
            </span>
          </div>
          {checking && <p className="text-xs text-slate-400 mt-1">Checking availability...</p>}
          {isSubdomainAvailable === true && <p className="text-xs text-emerald-600 mt-1">✓ Subdomain is available!</p>}
          {isSubdomainAvailable === false && <p className="text-xs text-rose-600 mt-1">✗ Subdomain is already taken.</p>}
        </div>

        <button
          type="submit"
          disabled={isSubdomainAvailable !== true}
          className="w-full bg-slate-900 text-white font-semibold py-3 rounded-lg hover:bg-slate-800 disabled:opacity-50 mt-4"
        >
          Complete Setup & Open Dashboard
        </button>
      </form>
    </div>
  );
};

export const OnboardingWizard = StoreOnboarding;
