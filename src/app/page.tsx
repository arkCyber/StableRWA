"use client";
// =====================================================================================
// File: src/app/page.tsx
// Description: Main page for the StableRWA web UI. Displays regular and blockchain assets,
//              and provides blockchain asset transfer functionality.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React from "react";

// Log import with timestamp
const importNow = new Date().toISOString();
// eslint-disable-next-line no-console
console.log(`[${importNow}] [page.tsx] Importing HomeClient`);

import HomeClient from "./HomeClient";

export default function Page() {
  const renderNow = new Date().toISOString();
  // eslint-disable-next-line no-console
  console.log(`[${renderNow}] [page.tsx] Rendering <Page> component, about to render <HomeClient />`);
  return <HomeClient />;
} 