// =====================================================================================
// File: src/app/HomeClient.tsx
// Description: Minimal HomeClient component for debugging Next.js entry. StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

"use client";

import React, { useEffect } from "react";

// Log import with timestamp
const importNow = new Date().toISOString();
// eslint-disable-next-line no-console
console.log(`[${importNow}] [HomeClient.tsx] Imported HomeClient`);

const HomeClient: React.FC = () => {
  useEffect(() => {
    const mountNow = new Date().toISOString();
    // eslint-disable-next-line no-console
    console.log(`[${mountNow}] [HomeClient.tsx] useEffect (componentDidMount)`);
  }, []);

  const renderNow = new Date().toISOString();
  // eslint-disable-next-line no-console
  console.log(`[${renderNow}] [HomeClient.tsx] Rendering <HomeClient> component`);

  return (
    <main className="max-w-2xl mx-auto py-8 space-y-8">
      <h1 className="text-3xl font-bold text-blue-700">StableRWA HomeClient is working!</h1>
      <p className="text-lg text-gray-700 mt-4">If you see this, the Next.js entry and client component are working.</p>
    </main>
  );
};

export default HomeClient; 