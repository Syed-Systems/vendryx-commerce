import React, { useState } from 'react';

export interface Role {
  id: string;
  name: string;
  description: string;
  policies: string[];
}

export const RbacManagement: React.FC = () => {
  const [roles, setRoles] = useState<Role[]>([
    {
      id: 'role_admin',
      name: 'Store Administrator',
      description: 'Full administrative access across all domain schemas',
      policies: ['*'],
    },
    {
      id: 'role_manager',
      name: 'Inventory Manager',
      description: 'Manage catalog, stock locations, and product taxonomy',
      policies: ['product:read', 'product:write', 'inventory:*'],
    },
    {
      id: 'role_support',
      name: 'Customer Support',
      description: 'View orders, process returns, and assist customers',
      policies: ['order:read', 'order:edit', 'customer:read'],
    },
  ]);

  return (
    <div className="rbac-container p-6 bg-white rounded-lg shadow-sm border">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-xl font-bold text-slate-900">Roles & Permissions (Clean-Room RBAC)</h1>
          <p className="text-sm text-slate-500">Fine-grained role-based access control without external enterprise locks.</p>
        </div>
        <button className="bg-slate-900 text-white px-4 py-2 rounded text-sm font-medium hover:bg-slate-800">
          + Create Custom Role
        </button>
      </div>

      <div className="grid gap-4">
        {roles.map((role) => (
          <div key={role.id} className="border p-4 rounded-lg flex justify-between items-center">
            <div>
              <h2 className="font-semibold text-slate-800">{role.name}</h2>
              <p className="text-sm text-slate-500">{role.description}</p>
              <div className="flex gap-2 mt-2">
                {role.policies.map((pol) => (
                  <span key={pol} className="bg-slate-100 text-slate-700 text-xs px-2 py-1 rounded">
                    {pol}
                  </span>
                ))}
              </div>
            </div>
            <button className="text-sm text-indigo-600 hover:text-indigo-800 font-medium">
              Edit Permissions
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
