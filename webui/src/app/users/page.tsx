'use client';

import { AppLayout } from '@/components/layout/app-layout';
import { UsersPage } from '@/components/users/users-page';

export default function Users() {
  return (
    <AppLayout>
      <UsersPage />
    </AppLayout>
  );
}
