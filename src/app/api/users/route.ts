// =====================================================================================
// File: src/app/api/users/route.ts
// Description: Mock API endpoint for fetching a list of users for the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { NextResponse } from "next/server";

export async function GET() {
  // Mock user data
  const users = [
    { id: "u1", name: "Alice" },
    { id: "u2", name: "Bob" },
    { id: "u3", name: "Charlie" },
  ];
  return NextResponse.json({ users });
} 