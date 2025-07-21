'use client';

import { AppLayout } from '@/components/layout/app-layout';
import { TransactionsPage } from '@/components/transactions/transactions-page';

export default function Transactions() {
  return (
    <AppLayout>
      <TransactionsPage />
    </AppLayout>
  );
}
