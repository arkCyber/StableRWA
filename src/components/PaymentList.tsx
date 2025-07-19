// =====================================================================================
// File: src/components/PaymentList.tsx
// Description: PaymentList component for displaying a list of payments in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React from "react";

export interface Payment {
  id: string;
  userId: string;
  amount: number;
  status: string;
  timestamp: string;
}

interface PaymentListProps {
  payments: Payment[];
  loading: boolean;
  error: string | null;
}

/**
 * PaymentList displays a list of payments with error and loading states.
 */
const PaymentList: React.FC<PaymentListProps> = ({ payments, loading, error }) => {
  if (loading) return <p className="text-gray-500">Loading payments...</p>;
  if (error) return <p className="text-red-500">Error: {error}</p>;
  if (payments.length === 0) return <p className="text-gray-500">No payments found.</p>;
  return (
    <ul className="space-y-2">
      {payments.map((payment) => (
        <li key={payment.id} className="p-2 border rounded bg-gray-100">
          <span className="font-medium">Payment ID: {payment.id}</span> | User: {payment.userId} | Amount: {payment.amount} | Status: {payment.status} | Time: {payment.timestamp}
        </li>
      ))}
    </ul>
  );
};

export default PaymentList; 