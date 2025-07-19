// =====================================================================================
// File: webui/src/app/page.tsx
// Description: Main page for the RWA Platform web UI. Displays regular and blockchain assets,
//              and provides blockchain asset transfer functionality.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

'use client';
import Image from "next/image";
import { useEffect, useState } from "react";
import BlockchainAssetList, { BlockchainAsset } from "../../../src/components/BlockchainAssetList";
import { fetchBlockchainAssets, transferBlockchainAsset } from "../../../src/services/blockchain";

interface Asset {
  id: string;
  name: string;
}

export default function Home() {
  // State for regular assets
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // State for blockchain assets
  const [bcAssets, setBcAssets] = useState<BlockchainAsset[]>([]);
  const [bcLoading, setBcLoading] = useState(true);
  const [bcError, setBcError] = useState<string | null>(null);
  const [transferLoading, setTransferLoading] = useState(false);

  // Fetch regular assets from backend
  useEffect(() => {
    async function fetchAssets() {
      setLoading(true);
      setError(null);
      try {
        const res = await fetch("http://127.0.0.1:8080/assets");
        if (!res.ok) throw new Error("Failed to fetch assets");
        const data = await res.json();
        setAssets(data.assets);
      } catch (err) {
        setError((err as Error).message);
      } finally {
        setLoading(false);
      }
    }
    fetchAssets();
  }, []);

  // Fetch blockchain assets from backend
  useEffect(() => {
    async function fetchBcAssets() {
      setBcLoading(true);
      setBcError(null);
      try {
        const assets = await fetchBlockchainAssets();
        setBcAssets(assets);
      } catch (err) {
        setBcError((err as Error).message);
      } finally {
        setBcLoading(false);
      }
    }
    fetchBcAssets();
  }, []);

  // Handler for blockchain asset transfer
  const handleTransfer = async (assetId: string, to: string) => {
    setTransferLoading(true);
    const now = new Date().toISOString();
    // eslint-disable-next-line no-console
    console.log(`[${now}] [Home] Initiating blockchain asset transfer: ${assetId} -> ${to}`);
    try {
      await transferBlockchainAsset(assetId, to);
      // eslint-disable-next-line no-console
      console.log(`[${now}] [Home] Blockchain asset transfer successful: ${assetId} -> ${to}`);
      // Refresh blockchain asset list after transfer
      setBcLoading(true);
      const assets = await fetchBlockchainAssets();
      setBcAssets(assets);
    } catch (err: any) {
      // eslint-disable-next-line no-console
      console.log(`[${now}] [Home] Blockchain asset transfer failed: ${err.message}`);
      throw err;
    } finally {
      setTransferLoading(false);
    }
  };

  return (
    <div className="font-sans grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20">
      <main className="flex flex-col gap-[32px] row-start-2 items-center sm:items-start w-full max-w-3xl">
        <Image
          className="dark:invert"
          src="/next.svg"
          alt="Next.js logo"
          width={180}
          height={38}
          priority
        />
        <ol className="font-mono list-inside list-decimal text-sm/6 text-center sm:text-left">
          <li className="mb-2 tracking-[-.01em]">
            Get started by editing{" "}
            <code className="bg-black/[.05] dark:bg-white/[.06] font-mono font-semibold px-1 py-0.5 rounded">
              src/app/page.tsx
            </code>
            .
          </li>
          <li className="tracking-[-.01em]">
            Save and see your changes instantly.
          </li>
        </ol>
        {/* Regular Asset List */}
        <section className="w-full">
          <h2 className="text-lg font-bold mb-2">Regular Assets</h2>
          {loading ? (
            <p className="text-gray-500">Loading assets...</p>
          ) : error ? (
            <p className="text-red-500">Error: {error}</p>
          ) : assets.length === 0 ? (
            <p className="text-gray-500">No assets found.</p>
          ) : (
            <ul className="space-y-2">
              {assets.map((asset) => (
                <li key={asset.id} className="p-2 border rounded bg-gray-50">
                  <span className="font-medium">{asset.name}</span> (ID: {asset.id})
                </li>
              ))}
            </ul>
          )}
        </section>
        {/* Blockchain Asset List */}
        <section className="w-full mt-8">
          <h2 className="text-lg font-bold mb-2">Blockchain Assets</h2>
          <BlockchainAssetList
            assets={bcAssets}
            loading={bcLoading}
            error={bcError}
            onTransfer={handleTransfer}
            transferLoading={transferLoading}
          />
        </section>
      </main>
      <footer className="row-start-3 flex gap-[24px] flex-wrap items-center justify-center">
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4"
          href="https://nextjs.org/learn?utm_source=create-next-app&utm_medium=appdir-template-tw&utm_campaign=create-next-app"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Image
            aria-hidden
            src="/file.svg"
            alt="File icon"
            width={16}
            height={16}
          />
          Learn
        </a>
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4"
          href="https://vercel.com/templates?framework=next.js&utm_source=create-next-app&utm_medium=appdir-template-tw&utm_campaign=create-next-app"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Image
            aria-hidden
            src="/window.svg"
            alt="Window icon"
            width={16}
            height={16}
          />
          Examples
        </a>
        <a
          className="flex items-center gap-2 hover:underline hover:underline-offset-4"
          href="https://nextjs.org?utm_source=create-next-app&utm_medium=appdir-template-tw&utm_campaign=create-next-app"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Image
            aria-hidden
            src="/globe.svg"
            alt="Globe icon"
            width={16}
            height={16}
          />
          Go to nextjs.org →
        </a>
      </footer>
    </div>
  );
} 