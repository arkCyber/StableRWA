// =====================================================================================
// File: src/components/BlockchainAssetList.tsx
// Description: BlockchainAssetList component for displaying and transferring blockchain assets in the StableRWA web UI.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import React from "react";
import { fetchBscBlockNumber } from "../services/asset";

export interface BlockchainAsset {
  id: string;
  name: string;
  type: string;
  value: number;
  chain: string; // e.g., Ethereum, Solana, Polkadot
  owner: string;
}

interface BlockchainAssetListProps {
  assets: BlockchainAsset[];
  loading: boolean;
  error: string | null;
  onTransfer: (assetId: string, to: string) => Promise<void>;
  transferLoading: boolean;
}

/**
 * BlockchainAssetList displays a list of blockchain assets and allows transfer actions.
 */
const BlockchainAssetList: React.FC<BlockchainAssetListProps> = ({ assets, loading, error, onTransfer, transferLoading }) => {
  const [transferTo, setTransferTo] = React.useState<{ [id: string]: string }>({});
  const [localError, setLocalError] = React.useState<string | null>(null);
  const [bscBlock, setBscBlock] = React.useState<number | null>(null);
  const [bscLoading, setBscLoading] = React.useState<boolean>(true);
  const [bscError, setBscError] = React.useState<string | null>(null);

  React.useEffect(() => {
    setBscLoading(true);
    fetchBscBlockNumber()
      .then((block) => {
        setBscBlock(block);
        setBscError(null);
      })
      .catch((e) => {
        setBscError(e.message || "Failed to fetch BSC block number");
        setBscBlock(null);
      })
      .finally(() => setBscLoading(false));
  }, []);

  const handleTransfer = async (id: string) => {
    setLocalError(null);
    const to = transferTo[id];
    if (!to) {
      setLocalError("Please enter a recipient address.");
      return;
    }
    const now = new Date().toISOString();
    // eslint-disable-next-line no-console
    console.log(`[${now}] [BlockchainAssetList] Initiating transfer of asset ${id} to ${to}`);
    try {
      await onTransfer(id, to);
      // eslint-disable-next-line no-console
      console.log(`[${now}] [BlockchainAssetList] Transfer successful for asset ${id}`);
    } catch (err: any) {
      setLocalError(err.message || "Transfer failed");
      // eslint-disable-next-line no-console
      console.log(`[${now}] [BlockchainAssetList] Transfer failed for asset ${id}: ${err.message}`);
    }
  };

  if (loading) return <p className="text-gray-500">Loading blockchain assets...</p>;
  if (error) return <p className="text-red-500">Error: {error}</p>;
  if (assets.length === 0) return <p className="text-gray-500">No blockchain assets found.</p>;
  return (
    <>
      <div className="mb-4">
        <span className="font-semibold">BSC Block Number: </span>
        {bscLoading ? (
          <span className="text-gray-500">Loading...</span>
        ) : bscError ? (
          <span className="text-red-500">Error: {bscError}</span>
        ) : (
          <span className="text-green-700">{bscBlock}</span>
        )}
      </div>
      <ul className="space-y-4">
        {assets.map((asset) => (
          <li key={asset.id} className="p-4 border rounded bg-gray-100">
            <div>
              <span className="font-medium">{asset.name}</span> (ID: {asset.id}) | Type: {asset.type} | Value: {asset.value} | Chain: {asset.chain} | Owner: {asset.owner}
            </div>
            <div className="mt-2 flex items-center space-x-2">
              <input
                type="text"
                className="border px-2 py-1 rounded"
                placeholder="Recipient address"
                value={transferTo[asset.id] || ""}
                onChange={(e) => setTransferTo({ ...transferTo, [asset.id]: e.target.value })}
                disabled={transferLoading}
              />
              <button
                className="bg-blue-600 text-white px-3 py-1 rounded disabled:opacity-50"
                onClick={() => handleTransfer(asset.id)}
                disabled={transferLoading}
              >
                {transferLoading ? "Transferring..." : "Transfer"}
              </button>
            </div>
          </li>
        ))}
        {localError && <li className="text-red-500">{localError}</li>}
      </ul>
    </>
  );
};

export default BlockchainAssetList; 