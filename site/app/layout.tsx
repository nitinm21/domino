import type { Metadata, Viewport } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-inter",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-mono",
});

export const metadata: Metadata = {
  title: "Domino — From meeting to merge.",
  description:
    "Domino records meetings inside Claude Code, transcribes it, and writes a grounded implementation plan you can execute.",
  openGraph: {
    title: "Domino — From meeting to merge.",
    description: "A meeting-aware thinking partner for Claude Code.",
    type: "website",
  },
};

export const viewport: Viewport = {
  themeColor: "#FDFDFC",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${inter.variable} ${mono.variable}`}>
      <body className="bg-paper font-sans text-ink">{children}</body>
    </html>
  );
}
