interface Props {
  provider: string;
  class?: string;
}

export default function ProviderIcon(props: Props) {
  const getProviderIcon = () => {
    switch (props.provider.toLowerCase()) {
      case 'openrouter':
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="orGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#6366F1", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#8B5CF6", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <path d="M5 12L9 8L9 10.5L15 10.5L15 13.5L9 13.5L9 16L5 12Z" fill="url(#orGradient)"/>
            <path d="M19 12L15 16L15 13.5L9 13.5L9 10.5L15 10.5L15 8L19 12Z" fill="url(#orGradient)" opacity="0.7"/>
            <circle cx="12" cy="12" r="2" fill="url(#orGradient)"/>
          </svg>
        );
      case 'openai':
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="oaGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#10A37F", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#0D7F5F", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.0462 6.0462 0 0 0 6.5144 2.9001A5.9835 5.9835 0 0 0 13.2599 24a6.0328 6.0328 0 0 0 5.7714-4.2058 5.9847 5.9847 0 0 0 3.9977-2.9001 6.0554 6.0554 0 0 0-.7471-7.0729Z" fill="url(#oaGradient)"/>
          </svg>
        );
      case 'anthropic':
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="anthropicGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#D97706", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#92400E", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <path d="M12 2L14.09 8.26L20.18 8.26L15.05 12.74L17.14 19L12 14.52L6.86 19L8.95 12.74L3.82 8.26L9.91 8.26L12 2Z" fill="url(#anthropicGradient)"/>
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
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="metaGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#1877F2", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#0C5FDC", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <path d="M12 2C6.48 2 2 6.48 2 12c0 4.99 3.66 9.12 8.44 9.88v-6.99H7.9v-2.89h2.54V9.4c0-2.5 1.49-3.89 3.78-3.89 1.09 0 2.23.2 2.23.2v2.46h-1.26c-1.24 0-1.63.77-1.63 1.56v1.88h2.78l-.44 2.89h-2.34v6.99C18.34 21.12 22 16.99 22 12c0-5.52-4.48-10-10-10z" fill="url(#metaGradient)"/>
          </svg>
        );
      case 'mistral':
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="mistralGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#FF6B35", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#F72B1C", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <path d="M12 2L4 7v10l8 5 8-5V7l-8-5z" fill="url(#mistralGradient)" opacity="0.8"/>
            <path d="M12 6L8 8.5v7l4 2.5 4-2.5v-7L12 6z" fill="url(#mistralGradient)"/>
          </svg>
        );
      case 'cohere':
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <defs>
              <linearGradient id="cohereGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style={{ "stop-color": "#FF6B6B", "stop-opacity": 1 }} />
                <stop offset="100%" style={{ "stop-color": "#C92A2A", "stop-opacity": 1 }} />
              </linearGradient>
            </defs>
            <circle cx="12" cy="12" r="10" fill="url(#cohereGradient)" opacity="0.2"/>
            <path d="M7 12a5 5 0 0 1 5-5v10a5 5 0 0 1-5-5z" fill="url(#cohereGradient)"/>
            <path d="M17 12a5 5 0 0 0-5-5v10a5 5 0 0 0 5-5z" fill="url(#cohereGradient)" opacity="0.7"/>
          </svg>
        );
      default:
        return (
          <svg viewBox="0 0 24 24" class={props.class}>
            <circle cx="12" cy="12" r="9" fill="var(--text-dim)" opacity="0.3"/>
            <path d="M9 12h6M12 9v6" stroke="var(--text-dim)" stroke-width="2" stroke-linecap="round"/>
          </svg>
        );
    }
  };

  return <>{getProviderIcon()}</>;
}