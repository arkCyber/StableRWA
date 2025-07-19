// =====================================================================================
// File: src/app/api/payments/route.ts
// Description: Mock API endpoint for fetching a list of payments for the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { NextResponse } from "next/server";

export async function GET() {
  // Mock payment data
  const payments = [
    { id: "p1", amount: 100.0, status: "Completed" },
    { id: "p2", amount: 250.5, status: "Pending" },
    { id: "p3", amount: 75.25, status: "Failed" },
  ];
  return NextResponse.json({ payments });
} 