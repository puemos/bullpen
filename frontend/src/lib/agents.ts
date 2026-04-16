export const INSTALL_URLS: Record<string, string> = {
  claude: "https://docs.anthropic.com/en/docs/claude-code",
  codex: "https://github.com/openai/codex",
  gemini: "https://github.com/google-gemini/gemini-cli",
  copilot: "https://docs.github.com/en/copilot",
  kiro: "https://kiro.dev",
  auggie: "https://augmentcode.com",
  junie: "https://www.jetbrains.com/junie",
  goose: "https://github.com/block/goose",
  qwen: "https://github.com/QwenLM/qwen-code",
  kimi: "https://kimi.ai",
  mistral: "https://docs.mistral.ai/getting-started/cli",
  opencode: "https://github.com/opencode-ai/opencode",
};

export function getLogoPath(agentName: string) {
  const lower = agentName.toLowerCase();
  if (lower.includes("claude")) return "/icons/claude.svg";
  if (lower.includes("gemini")) return "/icons/gemini.svg";
  if (lower.includes("codex")) return "/icons/codex.svg";
  if (lower.includes("qwen")) return "/icons/qwen.svg";
  if (lower.includes("kimi")) return "/icons/kimi.svg";
  if (lower.includes("mistral")) return "/icons/mistral.svg";
  if (lower.includes("grok")) return "/icons/grok.svg";
  if (lower.includes("copilot")) return "/icons/copilot.svg";
  if (lower.includes("kiro")) return "/icons/kiro.svg";
  if (lower.includes("auggie")) return "/icons/auggie.svg";
  if (lower.includes("junie")) return "/icons/junie.svg";
  if (lower.includes("goose")) return "/icons/goose.svg";
  return "/icons/opencode.svg";
}

export function sortAgentsByAvailability<T extends { available?: boolean }>(agents: T[]): T[] {
  return [...agents].sort((a, b) => {
    const aAvail = a.available !== false ? 0 : 1;
    const bAvail = b.available !== false ? 0 : 1;
    return aAvail - bAvail;
  });
}
