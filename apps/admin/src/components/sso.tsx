import React from 'react';

export interface SsoProvider {
  id: string;
  name: string;
  authUrl: string;
}

export const SsoLogin: React.FC = () => {
  const providers: SsoProvider[] = [
    { id: 'google', name: 'Sign in with Google Workspace', authUrl: '/api/auth/google' },
    { id: 'github', name: 'Sign in with GitHub', authUrl: '/api/auth/github' },
    { id: 'oidc', name: 'Enterprise OpenID Connect (OIDC)', authUrl: '/api/auth/oidc' },
  ];

  return (
    <div className="sso-card p-6 bg-white border rounded-xl max-w-md mx-auto shadow-sm">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-slate-900">Single Sign-On</h2>
        <p className="text-sm text-slate-500 mt-1">Authenticate via your identity provider</p>
      </div>

      <div className="space-y-3">
        {providers.map((p) => (
          <a
            key={p.id}
            href={p.authUrl}
            className="flex items-center justify-center w-full py-2.5 px-4 border rounded-lg text-sm font-semibold text-slate-700 hover:bg-slate-50 transition"
          >
            {p.name}
          </a>
        ))}
      </div>
    </div>
  );
};

export const SsoManagement = SsoLogin;
