import React, { useState } from 'react';

export interface ProductSummary {
  id: string;
  title: string;
  handle: string;
  status: 'draft' | 'proposed' | 'published' | 'rejected';
  variants_count: number;
}

export const ProductsView: React.FC = () => {
  const [products] = useState<ProductSummary[]>([
    { id: 'prod_1', title: 'Cotton Crewneck T-Shirt', handle: 'cotton-crewneck', status: 'published', variants_count: 3 },
    { id: 'prod_2', title: 'Heavyweight Fleece Hoodie', handle: 'heavyweight-hoodie', status: 'published', variants_count: 4 },
    { id: 'prod_3', title: 'Canvas Tote Bag', handle: 'canvas-tote', status: 'draft', variants_count: 1 },
  ]);
  const [searchTerm, setSearchTerm] = useState('');

  const filteredProducts = products.filter((p) =>
    p.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
    p.handle.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="products-container p-6 bg-white rounded-lg shadow-sm border space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-xl font-bold text-slate-900">Products Catalog</h1>
          <p className="text-sm text-slate-500">Manage store items, variants, pricing, and stock assignments.</p>
        </div>
        <div className="flex gap-3">
          <input
            type="text"
            placeholder="Search products..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="px-3 py-1.5 border rounded text-sm w-64 focus:outline-none focus:ring-1 focus:ring-slate-900"
          />
          <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
            + New Product
          </button>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm border-collapse">
          <thead>
            <tr className="border-b bg-slate-50 text-slate-600 font-semibold text-xs uppercase">
              <th className="p-3">Title</th>
              <th className="p-3">Handle</th>
              <th className="p-3">Status</th>
              <th className="p-3">Variants</th>
              <th className="p-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredProducts.map((prod) => (
              <tr key={prod.id} className="border-b hover:bg-slate-50">
                <td className="p-3 font-medium text-slate-900">{prod.title}</td>
                <td className="p-3 text-slate-500 font-mono text-xs">{prod.handle}</td>
                <td className="p-3">
                  <span
                    className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                      prod.status === 'published'
                        ? 'bg-emerald-50 text-emerald-700'
                        : 'bg-amber-50 text-amber-700'
                    }`}
                  >
                    {prod.status}
                  </span>
                </td>
                <td className="p-3 text-slate-600">{prod.variants_count} options</td>
                <td className="p-3 text-right">
                  <button className="text-indigo-600 hover:text-indigo-800 text-xs font-medium mr-3">
                    Edit
                  </button>
                  <button className="text-rose-600 hover:text-rose-800 text-xs font-medium">
                    Archive
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
