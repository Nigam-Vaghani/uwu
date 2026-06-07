export type AiProviderKind = "none" | "groq" | "gemini" | "ollama";

export type AiConfig = {
  provider: AiProviderKind;
  model: string;
  ollamaBaseUrl: string;
  hasApiKey: boolean;
};

export type SetAiConfigInput = {
  provider: AiProviderKind;
  model: string;
  ollamaBaseUrl: string;
  apiKey?: string;
};

export type AiConnectionTestResult = {
  ok: boolean;
  message: string;
  model: string;
};

export type AiCompleteResponse = {
  text: string;
  source: string;
};

export const AI_PROVIDER_OPTIONS: { value: AiProviderKind; label: string }[] = [
  { value: "none", label: "None (Rule-Based Only)" },
  { value: "groq", label: "Groq (Free)" },
  { value: "gemini", label: "Gemini Free Tier" },
  { value: "ollama", label: "Ollama (Local)" },
];

export const DEFAULT_AI_CONFIG: AiConfig = {
  provider: "groq",
  model: "llama3-8b-8192",
  ollamaBaseUrl: "http://localhost:11434",
  hasApiKey: false,
};

export function defaultModelForProvider(provider: AiProviderKind): string {
  switch (provider) {
    case "gemini":
      return "gemini-2.0-flash";
    case "ollama":
      return "llama3";
    case "groq":
      return "llama3-8b-8192";
    default:
      return "";
  }
}

export function providerRequiresApiKey(provider: AiProviderKind): boolean {
  return provider === "groq" || provider === "gemini";
}
