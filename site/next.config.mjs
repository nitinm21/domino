import { PHASE_DEVELOPMENT_SERVER } from "next/constants.js";

const sharedConfig = {
  reactStrictMode: true,
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
        ],
      },
    ];
  },
};

export default function nextConfig(phase) {
  return {
    ...sharedConfig,
    // Keep dev and build artifacts separate so `next build` doesn't corrupt a live `next dev`
    // session's chunk graph.
    distDir: phase === PHASE_DEVELOPMENT_SERVER ? ".next-dev" : ".next",
  };
}
