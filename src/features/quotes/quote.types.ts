export const BUILTIN_QUOTE_TYPES = ["funny", "motivational", "coding", "wisdom", "chill"] as const;

export type QuoteType = (typeof BUILTIN_QUOTE_TYPES)[number] | string;

export type QuoteConfig = {
  enabled: boolean;
  intervalMinutes: number;
  selectedType: string;
  customTypes: string[];
};

export type QuotePayload = {
  text: string;
  quoteType: string;
};

export const DEFAULT_QUOTE_CONFIG: QuoteConfig = {
  enabled: false,
  intervalMinutes: 5,
  selectedType: "motivational",
  customTypes: [],
};

export const QUOTE_EVENT = "quote:generated";
