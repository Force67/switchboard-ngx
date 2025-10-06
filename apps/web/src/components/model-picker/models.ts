export type ModelMeta = {
  id: string;
  name: string;
  tier?: 'pro' | 'free';
  disabled?: boolean;
  group?: 'gemini'|'gpt'|'other';
  badges: Array<'vision'|'tools'|'agent'|'image'>;
  pricing?: {
    input?: number;
    output?: number;
  };
};

export const MODELS: ModelMeta[] = [
  { id:'g25f', name:'Gemini 2.5 Flash', badges:['vision','tools'] },
  { id:'g25fl', name:'Gemini 2.5 Flash Lite', badges:['vision','tools'] },
  { id:'g25fi', name:'Gemini 2.5 Flash Image', tier:'pro', disabled:true, badges:['vision','image'] },
  { id:'g25p', name:'Gemini 2.5 Pro', tier:'pro', disabled:true, badges:['agent','vision','tools'] },
  { id:'gi4', name:'Gemini Imagen 4', tier:'pro', disabled:true, badges:['image'] },
  { id:'gi4u', name:'Gemini Imagen 4 Ultra', tier:'pro', disabled:true, badges:['image'] },
  { id:'gptimg', name:'GPT ImageGen', tier:'pro', disabled:true, badges:['vision','image'], group:'gpt' },
  { id:'gpt5', name:'GPT-5', disabled:true, badges:[] },
];