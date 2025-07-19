// =====================================================================================
// File: src/components/AssetList.tsx
// Description: AssetList component for displaying a list of assets in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React from "react";
import { Asset } from "../types/asset";

interface AssetListProps {
  assets: Asset[];
  loading: boolean;
  error: string | null;
}

/**
 * AssetList displays a list of assets with error and loading states.
 */
const AssetList: React.FC<AssetListProps> = ({ assets, loading, error }) => {
  if (loading) return <p className="text-gray-500">Loading assets...</p>;
  if (error) return <p className="text-red-500">Error: {error}</p>;
  if (assets.length === 0) return <p className="text-gray-500">No assets found.</p>;
  return (
    <ul className="space-y-2">
      {assets.map((asset) => (
        <li key={asset.id} className="p-2 border rounded bg-gray-100">
          <span className="font-medium">{asset.name}</span> (ID: {asset.id}) | Type: {asset.type} | Value: {asset.value}
        </li>
      ))}
    </ul>
  );
};

export default AssetList; 