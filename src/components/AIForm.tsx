// =====================================================================================
// File: src/components/AIForm.tsx
// Description: AIForm component for interacting with the AI service in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React, { useState } from "react";

interface AIFormProps {
  onSubmit: (prompt: string) => Promise<string>;
  loading: boolean;
}

/**
 * AIForm provides a form for sending prompts to the AI service and displaying the response.
 */
const AIForm: React.FC<AIFormProps> = ({ onSubmit, loading }) => {
  const [prompt, setPrompt] = useState("");
  const [response, setResponse] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setResponse(null);
    const now = new Date().toISOString();
    // eslint-disable-next-line no-console
    console.log(`[${now}] [AIForm] Sending prompt: ${prompt}`);
    try {
      const res = await onSubmit(prompt);
      setResponse(res);
      // eslint-disable-next-line no-console
      console.log(`[${now}] [AIForm] Received response: ${res}`);
    } catch (err: any) {
      setError(err.message || "AI request failed");
      // eslint-disable-next-line no-console
      console.log(`[${now}] [AIForm] AI request failed: ${err.message}`);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 p-4 border rounded bg-white max-w-xl mx-auto">
      <h2 className="text-xl font-bold mb-2">AI Assistant</h2>
      <div>
        <label className="block mb-1 font-medium">Prompt</label>
        <input
          type="text"
          className="w-full border px-2 py-1 rounded"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          required
        />
      </div>
      {error && <p className="text-red-500">{error}</p>}
      <button
        type="submit"
        className="w-full bg-purple-600 text-white py-2 rounded disabled:opacity-50"
        disabled={loading}
      >
        {loading ? "Asking..." : "Ask AI"}
      </button>
      {response && (
        <div className="mt-4 p-2 bg-gray-100 rounded">
          <strong>AI Response:</strong>
          <div className="whitespace-pre-wrap mt-1">{response}</div>
        </div>
      )}
    </form>
  );
};

export default AIForm; 