// =====================================================================================
// File: src/app/api/auth/login/route.ts
// Description: Mock API endpoint for user login authentication for the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { NextResponse } from "next/server";

export async function POST(request: Request) {
  const { username, password } = await request.json();
  // Simulate authentication logic
  if (username === "admin" && password === "password") {
    return NextResponse.json({ success: true, token: "mock-token-123", user: { username } });
  } else {
    return NextResponse.json({ success: false, error: "Invalid credentials" }, { status: 401 });
  }
} 