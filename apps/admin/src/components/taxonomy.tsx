import React, { useState } from 'react';

export interface CategoryNode {
  id: string;
  name: string;
  handle: string;
  children?: CategoryNode[];
}

export const TaxonomyManager: React.FC = () => {
  const [categories] = useState<CategoryNode[]>([
    {
      id: 'cat_apparel',
      name: 'Apparel',
      handle: 'apparel',
      children: [
        { id: 'cat_tops', name: 'Tops & T-Shirts', handle: 'tops' },
        { id: 'cat_hoodies', name: 'Hoodies & Sweatshirts', handle: 'hoodies' },
      ],
    },
    {
      id: 'cat_accessories',
      name: 'Accessories',
      handle: 'accessories',
      children: [
        { id: 'cat_bags', name: 'Bags & Totes', handle: 'bags' },
      ],
    },
  ]);

  return (
    <div className="taxonomy-container p-6 bg-white rounded-lg shadow-sm border space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-xl font-bold text-slate-900">Categories & Collections</h2>
          <p className="text-sm text-slate-500">Organize your store hierarchy and product tags.</p>
        </div>
        <button className="bg-slate-900 text-white px-3.5 py-1.5 rounded text-sm font-medium hover:bg-slate-800">
          + New Category
        </button>
      </div>

      <div className="border rounded-lg divide-y bg-slate-50">
        {categories.map((cat) => (
          <div key={cat.id} className="p-4 space-y-2">
            <div className="flex justify-between items-center">
              <span className="font-semibold text-slate-900 text-sm">{cat.name}</span>
              <span className="text-xs font-mono text-slate-500">/{cat.handle}</span>
            </div>
            {cat.children && (
              <div className="pl-4 border-l-2 border-slate-200 space-y-1 mt-2">
                {cat.children.map((child) => (
                  <div key={child.id} className="flex justify-between items-center py-1">
                    <span className="text-sm text-slate-700">{child.name}</span>
                    <span className="text-xs font-mono text-slate-400">/{child.handle}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
