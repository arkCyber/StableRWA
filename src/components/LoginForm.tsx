// =====================================================================================
// File: src/components/LoginForm.tsx
// Description: LoginForm component for user authentication in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React, { useState } from "react";

interface LoginFormProps {
  onLogin: (username: string, password: string) => Promise<void>;
  loading: boolean;
}

/**
 * LoginForm provides a form for user authentication with error handling and logging.
 */
const LoginForm: React.FC<LoginFormProps> = ({ onLogin, loading }) => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    const now = new Date().toISOString();
    // eslint-disable-next-line no-console
    console.log(`[${now}] [LoginForm] Attempting login for user: ${username}`);
    try {
      await onLogin(username, password);
      // eslint-disable-next-line no-console
      console.log(`[${now}] [LoginForm] Login successful for user: ${username}`);
    } catch (err: any) {
      setError(err.message || "Login failed");
      // eslint-disable-next-line no-console
      console.log(`[${now}] [LoginForm] Login failed for user: ${username} - ${err.message}`);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 p-4 border rounded bg-white max-w-sm mx-auto">
      <h2 className="text-xl font-bold mb-2">Login</h2>
      <div>
        <label className="block mb-1 font-medium">Username</label>
        <input
          type="text"
          className="w-full border px-2 py-1 rounded"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          required
        />
      </div>
      <div>
        <label className="block mb-1 font-medium">Password</label>
        <input
          type="password"
          className="w-full border px-2 py-1 rounded"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
      </div>
      {error && <p className="text-red-500">{error}</p>}
      <button
        type="submit"
        className="w-full bg-blue-600 text-white py-2 rounded disabled:opacity-50"
        disabled={loading}
      >
        {loading ? "Logging in..." : "Login"}
      </button>
    </form>
  );
};

export default LoginForm; 