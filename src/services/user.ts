// =====================================================================================
// File: src/services/user.ts
// Description: User API service for fetching user data from the backend microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { User } from "../components/UserList";

/**
 * Fetches the user list from the backend user microservice.
 * @returns Promise<User[]>
 */
export async function fetchUsers(endpoint = "http://localhost:8081/users"): Promise<User[]> {
  const res = await fetch(endpoint);
  if (!res.ok) throw new Error("Failed to fetch users");
  const data = await res.json();
  return data.users as User[];
} 