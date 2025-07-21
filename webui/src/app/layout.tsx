import type { Metadata } from "next";
import "./globals.css";
import { Providers } from "@/components/providers";

export const metadata: Metadata = {
  title: "StableRWA | Real World Asset Tokenization Platform",
  description: "Enterprise-grade platform for tokenizing real world assets on blockchain",
  keywords: ["StableRWA", "RWA", "tokenization", "blockchain", "real world assets", "DeFi"],
  authors: [{ name: "arkSong", email: "arksong2018@gmail.com" }],
  creator: "arkSong",
  publisher: "StableRWA",
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://stablerwa.com",
    title: "StableRWA | Real World Asset Tokenization Platform",
    description: "Enterprise-grade platform for tokenizing real world assets on blockchain",
    siteName: "StableRWA",
  },
  twitter: {
    card: "summary_large_image",
    title: "StableRWA | Real World Asset Tokenization Platform",
    description: "Enterprise-grade platform for tokenizing real world assets on blockchain",
    creator: "@stablerwa",
  },
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <head>
        <link rel="icon" href="/favicon.ico" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <meta name="theme-color" content="#020617" />
      </head>
      <body
        className="antialiased min-h-screen bg-background text-foreground font-sans"
        suppressHydrationWarning
      >
        <Providers>
          <div className="relative flex min-h-screen flex-col">
            {children}
          </div>
        </Providers>
      </body>
    </html>
  );
}
