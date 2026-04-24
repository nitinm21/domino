"use client";

import { createContext, useContext, useState, type ReactNode } from "react";

export type Tool = "claude-code" | "codex";

type ToolContextValue = {
  tool: Tool;
  setTool: (tool: Tool) => void;
};

const ToolContext = createContext<ToolContextValue | null>(null);

export function ToolProvider({ children }: { children: ReactNode }) {
  const [tool, setTool] = useState<Tool>("claude-code");
  return (
    <ToolContext.Provider value={{ tool, setTool }}>{children}</ToolContext.Provider>
  );
}

export function useTool() {
  const ctx = useContext(ToolContext);
  if (!ctx) throw new Error("useTool must be used within ToolProvider");
  return ctx;
}
