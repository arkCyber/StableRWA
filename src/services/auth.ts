// =====================================================================================
// File: src/services/auth.ts
// Description: Auth API service for user authentication with the backend microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

/**
 * Sends a login request to the backend auth microservice.
 * @param username - The user's username
 * @param password - The user's password
 * @returns Promise<void> (throws on error)
 */
export async function login(username: string, password: string, endpoint = "http://localhost:8083/login"): Promise<void> {
  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.message || "Login failed");
  }
} 