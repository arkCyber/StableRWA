'use client';

import { AppLayout } from '@/components/layout/app-layout';
import { AssetManagementPage } from '@/components/assets/asset-management-page';

export default function Assets() {
  return (
    <AppLayout>
      <AssetManagementPage />
    </AppLayout>
  );
}
