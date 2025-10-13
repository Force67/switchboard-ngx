export type ModelMeta = {
  id: string;
  name: string;
  tier?: 'pro' | 'free';
  disabled?: boolean;
  group?: 'gemini'|'gpt'|'other';
  provider?: string;
  badges: Array<'vision'|'tools'|'agent'|'image'|'reasoning'>;
  pricing?: {
    input?: number;
    output?: number;
  };
};

export const MODELS: ModelMeta[] = [];