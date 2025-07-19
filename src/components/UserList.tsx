// =====================================================================================
// File: src/components/UserList.tsx
// Description: UserList component for displaying a list of users in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React from "react";

export interface User {
  id: string;
  name: string;
}

interface UserListProps {
  users: User[];
  loading: boolean;
  error: string | null;
}

/**
 * UserList displays a list of users with error and loading states.
 */
const UserList: React.FC<UserListProps> = ({ users, loading, error }) => {
  if (loading) return <p className="text-gray-500">Loading users...</p>;
  if (error) return <p className="text-red-500">Error: {error}</p>;
  if (users.length === 0) return <p className="text-gray-500">No users found.</p>;
  return (
    <ul className="space-y-2">
      {users.map((user) => (
        <li key={user.id} className="p-2 border rounded bg-gray-100">
          <span className="font-medium">{user.name}</span> (ID: {user.id})
        </li>
      ))}
    </ul>
  );
};

export default UserList; 