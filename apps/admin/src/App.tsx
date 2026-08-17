import React, { useState } from 'react';
import { AuthScreen } from './components/auth';
import { ProductsView } from './components/products';
import { ProductMutationForm } from './components/product-mutations';
import { VariantsManager } from './components/variants';
import { TaxonomyManager } from './components/taxonomy';
import { InventoryManager } from './components/inventory';
import { PricingManager } from './components/pricing';
import { OrdersView } from './components/orders';
import { OrderEditPreview } from './components/order-edits';
import { ReturnsManagement } from './components/returns';
import { DraftOrdersView } from './components/draft-orders';
import { FulfillmentManagement } from './components/fulfillment';
import { CustomersView } from './components/customers';
import { MarketingPromotions } from './components/marketing';
import { AnalyticsDashboard } from './components/analytics';
import { SocialOrderQueue } from './components/social-queue';
import { SpreadsheetImport } from './components/spreadsheet-import';
import { WorkflowMonitor } from './components/workflows';
import { OnboardingWizard } from './components/onboarding';
import { RbacManagement } from './components/rbac';
import { SsoManagement } from './components/sso';
import { BillingSettings } from './components/billing';
import { FeatureFlagsOverview } from './components/feature-flags';
import { StoreSettings } from './components/settings';

export type NavItem =
  | 'analytics'
  | 'products'
  | 'product-mutations'
  | 'variants'
  | 'taxonomy'
  | 'inventory'
  | 'pricing'
  | 'orders'
  | 'order-edits'
  | 'returns'
  | 'draft-orders'
  | 'fulfillment'
  | 'customers'
  | 'marketing'
  | 'social-queue'
  | 'import'
  | 'workflows'
  | 'onboarding'
  | 'rbac'
  | 'sso'
  | 'billing'
  | 'feature-flags'
  | 'settings';

export const App: React.FC = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(true);
  const [activeTab, setActiveTab] = useState<NavItem>('analytics');

  if (!isAuthenticated) {
    return <AuthScreen onLoginSuccess={() => setIsAuthenticated(true)} />;
  }

  const renderContent = () => {
    switch (activeTab) {
      case 'analytics':
        return <AnalyticsDashboard />;
      case 'products':
        return <ProductsView />;
      case 'product-mutations':
        return <ProductMutationForm />;
      case 'variants':
        return <VariantsManager />;
      case 'taxonomy':
        return <TaxonomyManager />;
      case 'inventory':
        return <InventoryManager />;
      case 'pricing':
        return <PricingManager />;
      case 'orders':
        return <OrdersView />;
      case 'order-edits':
        return <OrderEditPreview />;
      case 'returns':
        return <ReturnsManagement />;
      case 'draft-orders':
        return <DraftOrdersView />;
      case 'fulfillment':
        return <FulfillmentManagement />;
      case 'customers':
        return <CustomersView />;
      case 'marketing':
        return <MarketingPromotions />;
      case 'social-queue':
        return <SocialOrderQueue />;
      case 'import':
        return <SpreadsheetImport />;
      case 'workflows':
        return <WorkflowMonitor />;
      case 'onboarding':
        return <OnboardingWizard />;
      case 'rbac':
        return <RbacManagement />;
      case 'sso':
        return <SsoManagement />;
      case 'billing':
        return <BillingSettings />;
      case 'feature-flags':
        return <FeatureFlagsOverview />;
      case 'settings':
        return <StoreSettings />;
      default:
        return <AnalyticsDashboard />;
    }
  };

  const navGroups = [
    {
      title: 'Commerce',
      items: [
        { id: 'analytics', label: 'Analytics' },
        { id: 'products', label: 'Products' },
        { id: 'product-mutations', label: 'Create Product' },
        { id: 'variants', label: 'Variants' },
        { id: 'taxonomy', label: 'Categories' },
        { id: 'inventory', label: 'Inventory' },
        { id: 'pricing', label: 'Pricing' },
      ],
    },
    {
      title: 'Operations',
      items: [
        { id: 'orders', label: 'Orders' },
        { id: 'order-edits', label: 'Order Edits' },
        { id: 'returns', label: 'Returns & Claims' },
        { id: 'draft-orders', label: 'Draft Orders' },
        { id: 'fulfillment', label: 'Fulfillment' },
        { id: 'social-queue', label: 'Social Queue' },
        { id: 'import', label: 'Import Catalog' },
        { id: 'workflows', label: 'Workflows' },
      ],
    },
    {
      title: 'Customers & Marketing',
      items: [
        { id: 'customers', label: 'Customers' },
        { id: 'marketing', label: 'Promotions' },
        { id: 'onboarding', label: 'Store Onboarding' },
      ],
    },
    {
      title: 'Administration',
      items: [
        { id: 'billing', label: 'Billing & Quotas' },
        { id: 'rbac', label: 'Roles (RBAC)' },
        { id: 'sso', label: 'SSO Config' },
        { id: 'feature-flags', label: 'Feature Flags' },
        { id: 'settings', label: 'Store Settings' },
      ],
    },
  ];

  return (
    <div className="flex h-screen bg-slate-100 font-sans text-slate-900 antialiased overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 bg-slate-900 text-slate-300 flex flex-col justify-between p-4 border-r border-slate-800">
        <div className="space-y-6 overflow-y-auto">
          <div className="flex items-center gap-2 px-2 py-1">
            <div className="w-8 h-8 rounded bg-gradient-to-tr from-indigo-500 to-purple-600 flex items-center justify-center font-bold text-white shadow">
              S
            </div>
            <div>
              <h1 className="font-bold text-white tracking-wide text-sm">STHENO STORE</h1>
              <p className="text-[10px] uppercase text-emerald-400 font-mono">Engine v0.1.0 • Pro</p>
            </div>
          </div>

          <nav className="space-y-4 text-xs">
            {navGroups.map((grp) => (
              <div key={grp.title} className="space-y-1">
                <p className="px-2 font-semibold text-slate-500 uppercase tracking-wider text-[10px]">
                  {grp.title}
                </p>
                {grp.items.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => setActiveTab(item.id as NavItem)}
                    className={`w-full text-left px-2.5 py-1.5 rounded transition-colors ${
                      activeTab === item.id
                        ? 'bg-slate-800 text-white font-medium shadow-sm'
                        : 'hover:bg-slate-800/50 hover:text-white'
                    }`}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            ))}
          </nav>
        </div>

        <div className="pt-4 border-t border-slate-800 px-2 space-y-2">
          <div className="flex items-center justify-between text-xs">
            <span className="text-slate-400">admin@stheno.store</span>
            <button
              onClick={() => setIsAuthenticated(false)}
              className="text-rose-400 hover:text-rose-300 font-medium"
            >
              Sign out
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Top bar */}
        <header className="h-14 bg-white border-b flex items-center justify-between px-8">
          <div className="flex items-center gap-4">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
              Dashboard / {activeTab.replace('-', ' ')}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <span className="inline-flex items-center gap-1.5 text-xs text-emerald-700 bg-emerald-50 px-2.5 py-1 rounded-full font-medium">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
              Postgres SAGA Engine: Healthy
            </span>
          </div>
        </header>

        {/* Dynamic View Container */}
        <div className="flex-1 overflow-y-auto p-8">
          <div className="max-w-6xl mx-auto">{renderContent()}</div>
        </div>
      </main>
    </div>
  );
};
