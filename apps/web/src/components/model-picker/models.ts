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

export const MODELS: ModelMeta[] = [
  { id:'g25f', name:'Gemini 2.5 Flash', provider:'google', badges:['vision','tools'] },
  { id:'g25fl', name:'Gemini 2.5 Flash Lite', provider:'google', badges:['vision','tools'] },
  { id:'g25fi', name:'Gemini 2.5 Flash Image', tier:'pro', disabled:true, provider:'google', badges:['vision','image'] },
  { id:'g25p', name:'Gemini 2.5 Pro', tier:'pro', disabled:true, provider:'google', badges:['agent','vision','tools'] },
  { id:'gi4', name:'Gemini Imagen 4', tier:'pro', disabled:true, provider:'google', badges:['image'] },
  { id:'gi4u', name:'Gemini Imagen 4 Ultra', tier:'pro', disabled:true, provider:'google', badges:['image'] },
  { id:'gptimg', name:'GPT ImageGen', tier:'pro', disabled:true, provider:'openai', badges:['vision','image'], group:'gpt' },
  { id:'gpt5', name:'GPT-5', disabled:true, provider:'openai', badges:[] },
];