import React, { useState } from 'react';

export const ProductMutationForm: React.FC = () => {
  const [title, setTitle] = useState('');
  const [handle, setHandle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState('published');
  const [saved, setSaved] = useState(false);

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    setSaved(true);
    setTimeout(() => setSaved(false), 3000);
  };

  return (
    <div className="product-mutation-container p-6 bg-white rounded-lg shadow-sm border space-y-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">Create / Edit Product</h2>
        <p className="text-sm text-slate-500">Define product details, handles, metadata, and default status.</p>
      </div>

      {saved && (
        <div className="p-3 bg-emerald-50 border border-emerald-200 text-emerald-800 text-sm rounded">
          Product saved successfully!
        </div>
      )}

      <form onSubmit={handleSave} className="space-y-4 max-w-xl">
        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Product Title
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
            className="w-full px-3 py-2 border rounded text-sm focus:outline-none focus:ring-1 focus:ring-slate-900"
            placeholder="e.g. Vintage Leather Jacket"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Handle / URL Slug
          </label>
          <input
            type="text"
            value={handle}
            onChange={(e) => setHandle(e.target.value)}
            className="w-full px-3 py-2 border rounded text-sm font-mono text-xs focus:outline-none focus:ring-1 focus:ring-slate-900"
            placeholder="vintage-leather-jacket"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Description
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            className="w-full px-3 py-2 border rounded text-sm focus:outline-none focus:ring-1 focus:ring-slate-900"
            placeholder="Detailed description of the product..."
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-700 uppercase mb-1">
            Publication Status
          </label>
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full px-3 py-2 border rounded text-sm bg-white focus:outline-none focus:ring-1 focus:ring-slate-900"
          >
            <option value="draft">Draft</option>
            <option value="proposed">Proposed</option>
            <option value="published">Published</option>
            <option value="rejected">Rejected</option>
          </select>
        </div>

        <button
          type="submit"
          className="bg-slate-900 text-white px-4 py-2 rounded text-sm font-medium hover:bg-slate-800"
        >
          Save Product
        </button>
      </form>
    </div>
  );
};
