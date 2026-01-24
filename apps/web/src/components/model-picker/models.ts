export type ModelMeta = {
  id: string;
  name: string;
  description?: string;
  tier?: 'pro' | 'free';
  disabled?: boolean;
  group?: 'gemini'|'gpt'|'anthropic'|'meta'|'mistral'|'other';
  provider?: string;
  badges: Array<'vision'|'tools'|'agent'|'image'|'reasoning'>;
  pricing?: {
    input?: number;
    output?: number;
  };
};

// Provider definitions with icons - supports any string, but these are known providers
export type ProviderIcon = string;

export type Provider = {
  id: string;
  name: string;
  icon: ProviderIcon;
};

export const PROVIDERS: Provider[] = [
  { id: 'favorites', name: 'Favorites', icon: 'star' },
  { id: 'openai', name: 'OpenAI', icon: 'openai' },
  { id: 'anthropic', name: 'Anthropic', icon: 'anthropic' },
  { id: 'google', name: 'Google', icon: 'google' },
  { id: 'meta-llama', name: 'Meta', icon: 'meta-llama' },
  { id: 'mistralai', name: 'Mistral', icon: 'mistralai' },
  { id: 'deepseek', name: 'DeepSeek', icon: 'deepseek' },
  { id: 'qwen', name: 'Qwen', icon: 'qwen' },
  { id: 'x-ai', name: 'xAI', icon: 'xai' },
  { id: 'cohere', name: 'Cohere', icon: 'cohere' },
  { id: 'perplexity', name: 'Perplexity', icon: 'perplexity' },
  { id: 'groq', name: 'Groq', icon: 'groq' },
  { id: 'nvidia', name: 'NVIDIA', icon: 'nvidia' },
  { id: 'other', name: 'Other', icon: 'other' },
];

// Sample models for demonstration
export const MODELS: ModelMeta[] = [
  // Google models
  {
    id: 'google/gemini-2.5-flash',
    name: 'Gemini 2.5 Flash',
    description: 'Lightning-fast with surprising capability',
    provider: 'google',
    group: 'gemini',
    badges: ['vision', 'tools', 'reasoning'],
    pricing: { input: 0.000075, output: 0.0003 },
  },
  {
    id: 'google/gemini-2.5-pro',
    name: 'Gemini 2.5 Pro',
    description: 'Google\'s newest flagship with advanced reasoning',
    provider: 'google',
    group: 'gemini',
    tier: 'pro',
    badges: ['vision', 'tools', 'reasoning', 'agent'],
    pricing: { input: 0.00125, output: 0.005 },
  },
  // OpenAI models
  {
    id: 'openai/gpt-4.1',
    name: 'GPT-4.1',
    description: 'Enhanced reasoning and instruction following',
    provider: 'openai',
    group: 'gpt',
    badges: ['vision', 'tools', 'reasoning'],
    pricing: { input: 0.002, output: 0.008 },
  },
  {
    id: 'openai/gpt-4.1-mini',
    name: 'GPT-4.1 Mini',
    description: 'Fast and cost-effective for everyday tasks',
    provider: 'openai',
    group: 'gpt',
    badges: ['vision', 'tools'],
    pricing: { input: 0.0001, output: 0.0004 },
  },
  {
    id: 'openai/o3-mini',
    name: 'o3-mini',
    description: 'Compact reasoning model',
    provider: 'openai',
    group: 'gpt',
    badges: ['reasoning', 'tools'],
    pricing: { input: 0.0011, output: 0.0044 },
  },
  // Anthropic models
  {
    id: 'anthropic/claude-sonnet-4',
    name: 'Claude Sonnet 4',
    description: 'Balanced intelligence and speed',
    provider: 'anthropic',
    group: 'anthropic',
    badges: ['vision', 'tools', 'reasoning', 'agent'],
    pricing: { input: 0.003, output: 0.015 },
  },
  {
    id: 'anthropic/claude-opus-4',
    name: 'Claude Opus 4',
    description: 'Most capable model for complex tasks',
    provider: 'anthropic',
    group: 'anthropic',
    tier: 'pro',
    badges: ['vision', 'tools', 'reasoning', 'agent'],
    pricing: { input: 0.015, output: 0.075 },
  },
  {
    id: 'anthropic/claude-haiku-3.5',
    name: 'Claude Haiku 3.5',
    description: 'Ultra-fast responses for simple tasks',
    provider: 'anthropic',
    group: 'anthropic',
    badges: ['vision', 'tools'],
    pricing: { input: 0.0008, output: 0.004 },
  },
  // Meta models
  {
    id: 'meta/llama-4-maverick',
    name: 'Llama 4 Maverick',
    description: 'Open-weight frontier model',
    provider: 'meta',
    group: 'meta',
    badges: ['vision', 'tools', 'reasoning'],
    pricing: { input: 0.0002, output: 0.0006 },
  },
  {
    id: 'meta/llama-4-scout',
    name: 'Llama 4 Scout',
    description: 'Efficient multi-modal reasoning',
    provider: 'meta',
    group: 'meta',
    badges: ['vision', 'tools'],
    pricing: { input: 0.00015, output: 0.0002 },
  },
  // Mistral models
  {
    id: 'mistral/mistral-large-2',
    name: 'Mistral Large 2',
    description: 'Top-tier multilingual performance',
    provider: 'mistral',
    group: 'mistral',
    badges: ['tools', 'reasoning', 'agent'],
    pricing: { input: 0.002, output: 0.006 },
  },
  {
    id: 'mistral/codestral',
    name: 'Codestral',
    description: 'Optimized for code generation',
    provider: 'mistral',
    group: 'mistral',
    badges: ['tools', 'agent'],
    pricing: { input: 0.0003, output: 0.0009 },
  },
  // DeepSeek
  {
    id: 'deepseek/deepseek-r1',
    name: 'DeepSeek R1',
    description: 'Advanced reasoning at low cost',
    provider: 'deepseek',
    group: 'other',
    badges: ['reasoning', 'tools'],
    pricing: { input: 0.00055, output: 0.00219 },
  },
  {
    id: 'deepseek/deepseek-v3',
    name: 'DeepSeek V3',
    description: 'Balanced performance and efficiency',
    provider: 'deepseek',
    group: 'other',
    badges: ['vision', 'tools'],
    pricing: { input: 0.00027, output: 0.0011 },
  },
  // Qwen
  {
    id: 'qwen/qwen-2.5-72b',
    name: 'Qwen 2.5 72B',
    description: 'Powerful open model from Alibaba',
    provider: 'qwen',
    group: 'other',
    badges: ['vision', 'tools', 'reasoning'],
    pricing: { input: 0.00035, output: 0.0004 },
  },
];
