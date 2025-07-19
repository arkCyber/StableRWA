// =====================================================================================
// File: src/services/ai.ts
// Description: AI API service for interacting with the backend AI microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

/**
 * Sends a prompt to the backend AI microservice and returns the response.
 * @param prompt - The user's prompt
 * @returns Promise<string> (AI response)
 */
export async function askAI(prompt: string, endpoint = "http://localhost:8085/ai"): Promise<string> {
  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ prompt }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.message || "AI request failed");
  }
  const data = await res.json();
  return data.response as string;
} 