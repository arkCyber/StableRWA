// =====================================================================================
// File: src/app/api/assets/route.ts
// Description: Mock API endpoint for fetching a list of assets for the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { NextResponse } from "next/server";

export async function GET() {
  // Mock asset data
  const assets = [
    { id: "1", name: "Building A" },
    { id: "2", name: "Land Parcel B" },
    { id: "3", name: "Vehicle C" },
  ];
  return NextResponse.json({ assets });
} 