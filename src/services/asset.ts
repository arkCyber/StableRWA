// =====================================================================================
// File: src/services/asset.ts
// Description: Asset API service for fetching asset data from the backend microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { Asset } from "../types/asset";

/**
 * Fetches the asset list from the backend asset microservice.
 * @returns Promise<Asset[]>
 */
export async function fetchAssets(endpoint = "http://localhost:8084/assets"): Promise<Asset[]> {
  const res = await fetch(endpoint);
  if (!res.ok) throw new Error("Failed to fetch assets");
  const data = await res.json();
  return data.assets as Asset[];
}

/**
 * Fetches the latest Binance Smart Chain block number from the backend asset microservice.
 * @returns Promise<number>
 */
export async function fetchBscBlockNumber(endpoint = "http://localhost:8080/bsc-block-number"): Promise<number> {
  const res = await fetch(endpoint);
  if (!res.ok) throw new Error("Failed to fetch BSC block number");
  const data = await res.json();
  return data.block_number as number;
} 