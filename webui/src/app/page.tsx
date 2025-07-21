// =====================================================================================
// File: webui/src/app/page.tsx
// Description: Main dashboard page for the RWA Platform web UI
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

'use client';

import { Dashboard } from '@/components/dashboard/dashboard';
import { AppLayout } from '@/components/layout/app-layout';

export default function Home() {
  return (
    <AppLayout>
      <Dashboard />
    </AppLayout>
  );
}