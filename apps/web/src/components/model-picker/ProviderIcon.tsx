interface Props {
  provider: string;
  class?: string;
}

export default function ProviderIcon(props: Props) {
  const key = () => props.provider.toLowerCase();

  // Star icon for favorites
  if (key() === 'star' || key() === 'favorites') {
    return (
      <svg viewBox="0 0 24 24" class={props.class} fill="none">
        <path
          d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
          fill="#FFD700"
          stroke="#FFD700"
          stroke-width="1"
        />
      </svg>
    );
  }

  // Provider-specific icons
  switch (key()) {
    case 'openai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#10A37F">
          <path d="M22.28 9.82a5.98 5.98 0 0 0-.52-4.91 6.05 6.05 0 0 0-6.51-2.9A6.07 6.07 0 0 0 4.98 4.18a5.98 5.98 0 0 0-4 2.9 6.05 6.05 0 0 0 .74 7.1 5.98 5.98 0 0 0 .51 4.91 6.05 6.05 0 0 0 6.51 2.9A5.98 5.98 0 0 0 13.26 24a6.03 6.03 0 0 0 5.77-4.21 5.98 5.98 0 0 0 4-2.9 6.06 6.06 0 0 0-.75-7.07zM13.26 22.43a4.48 4.48 0 0 1-2.88-1.04l.14-.08 4.78-2.76a.77.77 0 0 0 .39-.68v-6.74l2.02 1.17a.07.07 0 0 1 .04.05v5.58a4.5 4.5 0 0 1-4.49 4.5zM3.6 18.3a4.47 4.47 0 0 1-.54-3.01l.14.08 4.78 2.76a.77.77 0 0 0 .78 0l5.84-3.37v2.33a.08.08 0 0 1-.03.06l-4.83 2.79a4.5 4.5 0 0 1-6.14-1.64zM2.34 7.9a4.48 4.48 0 0 1 2.37-1.97V11.6a.77.77 0 0 0 .39.68l5.81 3.35-2.02 1.17a.08.08 0 0 1-.07 0L4 14.01a4.5 4.5 0 0 1-1.66-6.11zm16.1 3.86l-5.84-3.37 2.02-1.16a.08.08 0 0 1 .07 0l4.83 2.79a4.49 4.49 0 0 1-.68 8.1v-5.68a.77.77 0 0 0-.4-.68zm2.01-3.02l-.14-.09-4.77-2.78a.78.78 0 0 0-.79 0L9.41 9.23V6.9a.07.07 0 0 1 .03-.06l4.83-2.79a4.5 4.5 0 0 1 6.68 4.66zM8.31 12.86l-2.02-1.16a.07.07 0 0 1-.04-.06V6.07a4.5 4.5 0 0 1 7.38-3.45l-.14.08-4.78 2.76a.77.77 0 0 0-.39.68zm1.1-2.37l2.6-1.5 2.6 1.5v3l-2.6 1.5-2.6-1.5z"/>
        </svg>
      );

    case 'anthropic':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#D4A27F">
          <path d="M13.83 3.5h3.6L24 20.5h-3.6l-6.57-17zm-7.26 0h3.77L16.9 20.5h-3.67l-1.34-3.46H5.02l-1.35 3.46H0L6.57 3.5zm2.33 4.39l-2.39 6.16h4.78l-2.39-6.16z"/>
        </svg>
      );

    case 'google':
      return (
        <svg viewBox="0 0 24 24" class={props.class}>
          <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
          <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
          <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
          <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
        </svg>
      );

    case 'meta':
    case 'meta-llama':
      // Meta logo - clean infinity symbol
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <path d="M4.5 12c0-2.5 1.5-5 4-5 1.5 0 2.5 1 3.5 2.5l.5.8.5-.8c1-1.5 2-2.5 3.5-2.5 2.5 0 4 2.5 4 5s-1.5 5-4 5c-1.5 0-2.5-1-3.5-2.5l-.5-.8-.5.8c-1 1.5-2 2.5-3.5 2.5-2.5 0-4-2.5-4-5z" stroke="#0081FB" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'mistral':
    case 'mistralai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <rect x="2" y="2" width="5" height="5" fill="#F7D046"/>
          <rect x="17" y="2" width="5" height="5" fill="#232323"/>
          <rect x="2" y="9.5" width="5" height="5" fill="#F7D046"/>
          <rect x="9.5" y="9.5" width="5" height="5" fill="#F7931A"/>
          <rect x="17" y="9.5" width="5" height="5" fill="#F7931A"/>
          <rect x="2" y="17" width="5" height="5" fill="#F7D046"/>
          <rect x="17" y="17" width="5" height="5" fill="#232323"/>
        </svg>
      );

    case 'deepseek':
      // DeepSeek - blue whale/dolphin inspired
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#4D6BFE"/>
          <path d="M6 11c0-3 2.5-5 6-5 4 0 6 3 6 5 0 3-2 5-6 6-2.5.6-4-.5-5-2" stroke="white" stroke-width="2" stroke-linecap="round" fill="none"/>
          <circle cx="9" cy="10" r="1.5" fill="white"/>
        </svg>
      );

    case 'qwen':
    case 'alibaba':
      // Qwen - purple/blue with stylized Q shape
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#6149f6"/>
          <circle cx="12" cy="11" r="5" stroke="white" stroke-width="2" fill="none"/>
          <path d="M15 14l3 4" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'x-ai':
    case 'xai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="currentColor">
          <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
        </svg>
      );

    case 'cohere':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#39594D"/>
          <path d="M8 12a4 4 0 0 1 4-4" stroke="#D18EE2" stroke-width="3" stroke-linecap="round"/>
          <path d="M16 12a4 4 0 0 1-4 4" stroke="white" stroke-width="3" stroke-linecap="round"/>
        </svg>
      );

    case 'perplexity':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <rect x="2" y="2" width="20" height="20" rx="4" fill="#1B1F23"/>
          <path d="M12 5v14M5 12h14M7 7l10 10M17 7L7 17" stroke="#20B2AA" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'groq':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#F55036"/>
          <path d="M8 12h8M12 8v8" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
      );

    case 'together':
    case 'togetherai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#0066FF">
          <circle cx="7" cy="7" r="3"/>
          <circle cx="17" cy="7" r="3"/>
          <circle cx="7" cy="17" r="3"/>
          <circle cx="17" cy="17" r="3"/>
          <rect x="9.5" y="6" width="5" height="2"/>
          <rect x="6" y="9.5" width="2" height="5"/>
          <rect x="16" y="9.5" width="2" height="5"/>
          <rect x="9.5" y="16" width="5" height="2"/>
        </svg>
      );

    case 'nvidia':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#76B900">
          <path d="M8.948 8.798c-2.492.168-4.237 1.792-4.237 1.792s2.058-2.718 5.644-2.99l-.076-.153c-.186-.373-.41-.694-.685-.96C8.18 5.253 6.003 5.56 6.003 5.56s.636-.074 1.345-.032c.717.042 1.516.194 2.316.574 1.26.597 2.14 1.654 2.708 2.88l.078.158s.293-.04.695-.04c.506 0 1.136.08 1.79.31 1.066.377 2.06 1.202 2.645 2.618.584 1.416.52 3.45-.682 5.322 0 0 1.088-1.652.957-3.93-.13-2.28-1.324-3.78-1.324-3.78s.596.768.89 1.9c.295 1.132.19 2.41-.44 3.593-.63 1.183-1.798 2.208-3.472 2.552-1.675.344-3.283-.168-4.448-1.03-1.166-.862-1.94-2.127-2.185-3.628-.244-1.5.096-3.144 1.108-4.42 1.01-1.276 2.588-2.125 4.377-2.23z"/>
        </svg>
      );

    case 'amazon':
    case 'aws':
    case 'bedrock':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#FF9900">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z"/>
          <path d="M12 6l-6 4.5 2 1.5 4-3 4 3 2-1.5L12 6zM6 13.5l6 4.5 6-4.5-2-1.5-4 3-4-3-2 1.5z"/>
        </svg>
      );

    case 'huggingface':
      return (
        <svg viewBox="0 0 24 24" class={props.class}>
          <circle cx="12" cy="12" r="10" fill="#FFD21E"/>
          <circle cx="8.5" cy="10" r="1.5" fill="#1a1a1a"/>
          <circle cx="15.5" cy="10" r="1.5" fill="#1a1a1a"/>
          <path d="M8 14c0 2.21 1.79 4 4 4s4-1.79 4-4" stroke="#1a1a1a" stroke-width="1.5" stroke-linecap="round" fill="none"/>
        </svg>
      );

    case 'microsoft':
    case 'azure':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <rect x="2" y="2" width="9" height="9" fill="#F25022"/>
          <rect x="13" y="2" width="9" height="9" fill="#7FBA00"/>
          <rect x="2" y="13" width="9" height="9" fill="#00A4EF"/>
          <rect x="13" y="13" width="9" height="9" fill="#FFB900"/>
        </svg>
      );

    case 'ai21':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#9C27B0"/>
          <path d="M8 16l4-10 4 10M9.5 13h5" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'minimax':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#6C5CE7"/>
          <path d="M6 15V9l3 3 3-3 3 3 3-3v6" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'zhipu':
    case 'glm':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#1E88E5"/>
          <path d="M7 12h10M12 7v10" stroke="white" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
      );

    case 'moonshot':
    case 'moonshotai':
    case 'kimi':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#1a1a2e"/>
          <path d="M12 6a6 6 0 0 1 0 12 4 4 0 0 0 0-12z" fill="#f4f4f4"/>
        </svg>
      );

    case '01-ai':
    case 'yi':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#00D4AA"/>
          <path d="M9 8v8M12 6v12M15 8v8" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'inflection':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#FF6B35"/>
          <circle cx="12" cy="8" r="2" fill="white"/>
          <path d="M12 12v6" stroke="white" stroke-width="3" stroke-linecap="round"/>
        </svg>
      );

    case 'nousresearch':
    case 'nous':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#8B5CF6"/>
          <path d="M8 16V8l8 8V8" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'liquid':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#06B6D4"/>
          <path d="M12 6c-3 0-5 4-5 6s2 6 5 6 5-4 5-6-2-6-5-6z" fill="white" opacity="0.9"/>
        </svg>
      );

    case 'databricks':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <path d="M12 2L2 7.5l10 5.5 10-5.5L12 2z" fill="#FF3621"/>
          <path d="M2 12l10 5.5L22 12" stroke="#FF3621" stroke-width="2"/>
          <path d="M2 16.5L12 22l10-5.5" stroke="#FF3621" stroke-width="2"/>
        </svg>
      );

    case 'openrouter':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#6366F1"/>
          <path d="M7 12h10M10 9l-3 3 3 3M14 9l3 3-3 3" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    // Additional OpenRouter providers
    case 'aion-labs':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#3B82F6"/>
          <circle cx="12" cy="12" r="4" stroke="white" stroke-width="2" fill="none"/>
          <circle cx="12" cy="12" r="1.5" fill="white"/>
        </svg>
      );

    case 'allenai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#2563EB"/>
          <path d="M8 16l4-10 4 10" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          <path d="M9.5 13h5" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'alpindale':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#10B981"/>
          <path d="M12 6l6 12H6l6-12z" fill="white"/>
        </svg>
      );

    case 'anthracite-org':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#374151"/>
          <path d="M8 8h8v8H8z" stroke="white" stroke-width="2"/>
        </svg>
      );

    case 'arcee-ai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#8B5CF6"/>
          <path d="M12 6a6 6 0 0 1 6 6M12 6a6 6 0 0 0-6 6" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="12" r="2" fill="white"/>
        </svg>
      );

    case 'baidu':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#2319DC">
          <path d="M5.5 8.5c0-1.5 1-3 2.5-3s2.5 1.5 2.5 3-1 3-2.5 3-2.5-1.5-2.5-3zM13.5 8.5c0-1.5 1-3 2.5-3s2.5 1.5 2.5 3-1 3-2.5 3-2.5-1.5-2.5-3zM9 16c0-2 1.5-3.5 3-3.5s3 1.5 3 3.5-1.5 3.5-3 3.5-3-1.5-3-3.5z"/>
        </svg>
      );

    case 'bytedance':
    case 'bytedance-seed':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#00F5D4"/>
          <path d="M8 8v8l8-4-8-4z" fill="#1a1a1a"/>
        </svg>
      );

    case 'cognitivecomputations':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#EC4899"/>
          <circle cx="12" cy="12" r="4" stroke="white" stroke-width="2" fill="none"/>
          <path d="M12 2v4M12 18v4M2 12h4M18 12h4" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'deepcogito':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#7C3AED"/>
          <path d="M8 12c0-2.2 1.8-4 4-4s4 1.8 4 4" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="14" r="2" fill="white"/>
        </svg>
      );

    case 'eleutherai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#059669"/>
          <path d="M7 12h10M12 7v10" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'essentialai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#F59E0B"/>
          <circle cx="12" cy="12" r="5" stroke="white" stroke-width="2" fill="none"/>
        </svg>
      );

    case 'gryphe':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#DC2626"/>
          <path d="M12 6c3 0 5 2 5 4 0 3-2 5-5 8-3-3-5-5-5-8 0-2 2-4 5-4z" fill="white"/>
        </svg>
      );

    case 'ibm-granite':
    case 'ibm':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#0F62FE">
          <path d="M4 4h16v4H4zM4 10h8v4H4zM4 16h16v4H4zM14 10h6v4h-6z"/>
        </svg>
      );

    case 'inception':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#4F46E5"/>
          <rect x="8" y="8" width="8" height="8" rx="1" stroke="white" stroke-width="2" fill="none"/>
          <rect x="10" y="10" width="4" height="4" fill="white"/>
        </svg>
      );

    case 'kwaipilot':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#FF5722"/>
          <path d="M8 8l8 8M16 8l-8 8" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'mancer':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#9333EA"/>
          <path d="M12 6v12M8 10l4-4 4 4" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'meituan':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#FFD000"/>
          <path d="M8 12a4 4 0 0 1 8 0 4 4 0 0 1-8 0z" fill="#1a1a1a"/>
        </svg>
      );

    case 'morph':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#14B8A6"/>
          <path d="M7 12c0-3 2-5 5-5s5 2 5 5-2 5-5 5" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'neversleep':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#1e1b4b"/>
          <circle cx="12" cy="12" r="4" fill="#a855f7"/>
          <path d="M12 4v2M12 18v2M4 12h2M18 12h2" stroke="#a855f7" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'nex-agi':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#00CED1"/>
          <path d="M12 6l6 10H6l6-10z" stroke="white" stroke-width="2" fill="none"/>
        </svg>
      );

    case 'opengvlab':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#3B82F6"/>
          <circle cx="12" cy="12" r="3" fill="white"/>
          <path d="M12 5v2M12 17v2M5 12h2M17 12h2" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'prime-intellect':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#6366F1"/>
          <path d="M12 7v10M9 10h6" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'raifle':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#EF4444"/>
          <path d="M12 8v8M8 12h8" stroke="white" stroke-width="3" stroke-linecap="round"/>
        </svg>
      );

    case 'relace':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#8B5CF6"/>
          <path d="M8 8l8 8M8 16l8-8" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'sao10k':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#EC4899"/>
          <path d="M7 12a5 5 0 0 1 10 0" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="14" r="2" fill="white"/>
        </svg>
      );

    case 'stepfun-ai':
    case 'stepfun':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#22C55E"/>
          <path d="M6 14h4v4M10 14v-4h4M14 10V6h4" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'switchpoint':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#0EA5E9"/>
          <circle cx="9" cy="9" r="2" fill="white"/>
          <circle cx="15" cy="15" r="2" fill="white"/>
          <path d="M10.5 10.5l3 3" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'tencent':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#12B7F5"/>
          <ellipse cx="9" cy="10" rx="2" ry="3" fill="white"/>
          <ellipse cx="15" cy="10" rx="2" ry="3" fill="white"/>
          <path d="M8 15c2 2 6 2 8 0" stroke="white" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
      );

    case 'thedrummer':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#F97316"/>
          <circle cx="8" cy="12" r="2" fill="white"/>
          <circle cx="16" cy="12" r="2" fill="white"/>
          <path d="M10 12h4" stroke="white" stroke-width="2"/>
        </svg>
      );

    case 'tngtech':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#0891B2"/>
          <path d="M6 8h12M12 8v10" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'undi95':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#A855F7"/>
          <path d="M8 8v6a4 4 0 0 0 8 0V8" stroke="white" stroke-width="2" stroke-linecap="round"/>
        </svg>
      );

    case 'writer':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#1a1a1a"/>
          <path d="M7 17l3-10 2 6 2-6 3 10" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'xiaomi':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="#FF6900">
          <path d="M4 4h16v16H4V4zm2 2v12h12V6H6z"/>
          <path d="M8 8h3v8H8V8zm5 0h3v8h-3V8z"/>
        </svg>
      );

    case 'z-ai':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#3B82F6"/>
          <path d="M7 8h10l-10 8h10" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      );

    case 'alfredpros':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#7C3AED"/>
          <path d="M12 6l6 12H6l6-12z" stroke="white" stroke-width="2" fill="none"/>
        </svg>
      );

    case 'sophosympatheia':
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="10" fill="#EC4899"/>
          <path d="M12 7c-3 0-5 2.5-5 5s2 5 5 5" stroke="white" stroke-width="2" stroke-linecap="round"/>
          <circle cx="14" cy="12" r="2" fill="white"/>
        </svg>
      );

    default:
      // Fallback generic icon
      return (
        <svg viewBox="0 0 24 24" class={props.class} fill="none">
          <circle cx="12" cy="12" r="9" stroke="var(--text-dim)" stroke-width="1.5" opacity="0.5"/>
          <circle cx="12" cy="12" r="4" fill="var(--text-dim)" opacity="0.5"/>
        </svg>
      );
  }
}
