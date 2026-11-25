// Copy logo files from this kit to /public/branding/ and icon PNGs to /public/icons/.
// Next.js App Router metadata example (adjust URLs to your deployed domain).
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Vendryx | Commerce OS by Syed Systems',
  description: 'A modern commerce operating system for social and independent merchants.',
  icons: {
    icon: [{ url: '/branding/favicon.svg', type: 'image/svg+xml' }, { url: '/icons/favicon-32x32.png', sizes: '32x32' }],
    apple: [{ url: '/icons/app-icon-dark-180.png', sizes: '180x180' }],
  },
  openGraph: {
    title: 'Vendryx | Commerce OS',
    description: 'Build. Sell. Scale.',
    images: [{ url: '/branding/open-graph.png', width: 1200, height: 630 }],
  },
};
