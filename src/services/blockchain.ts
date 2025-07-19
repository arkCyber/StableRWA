// =====================================================================================
// File: src/services/blockchain.ts
// Description: Blockchain API service for interacting with the backend blockchain microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { BlockchainAsset } from "../components/BlockchainAssetList";

/**
 * Fetches the blockchain asset list from the backend blockchain microservice.
 * @returns Promise<BlockchainAsset[]>
 */
export async function fetchBlockchainAssets(endpoint = "http://localhost:8086/blockchain-assets"): Promise<BlockchainAsset[]> {
  const res = await fetch(endpoint);
  if (!res.ok) throw new Error("Failed to fetch blockchain assets");
  const data = await res.json();
  return data.assets as BlockchainAsset[];
}

/**
 * Transfers a blockchain asset to another address via the backend blockchain microservice.
 * @param assetId - The asset ID
 * @param to - The recipient address
 * @returns Promise<void>
 */
export async function transferBlockchainAsset(assetId: string, to: string, endpoint = "http://localhost:8086/transfer"): Promise<void> {
  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ assetId, to }),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.message || "Transfer failed");
  }
} 