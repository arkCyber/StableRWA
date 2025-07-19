// =====================================================================================
// File: src/types/payment.ts
// Description: Global TypeScript type definition for Payment in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

export interface Payment {
  id: string;
  userId: string;
  amount: number;
  status: string;
  timestamp: string;
} 