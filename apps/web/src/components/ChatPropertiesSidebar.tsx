import { Accessor, Setter, Show, createSignal, createEffect } from "solid-js";

export interface ChatProperties {
  webSearchEnabled: boolean;
  temperature?: number;
  maxTokens?: number;
  systemPrompt?: string;
}

interface Props {
  isOpen: Accessor<boolean>;
  onClose: () => void;
  properties: Accessor<ChatProperties>;
  setProperties: Setter<ChatProperties>;
  currentModelId?: Accessor<string | null>;
}

export default function ChatPropertiesSidebar(props: Props) {
  const [localTemp, setLocalTemp] = createSignal(0.7);
  const [localMaxTokens, setLocalMaxTokens] = createSignal(4096);
  const [localSystemPrompt, setLocalSystemPrompt] = createSignal("");

  // Sync local state with props
  createEffect(() => {
    const p = props.properties();
    setLocalTemp(p.temperature ?? 0.7);
    setLocalMaxTokens(p.maxTokens ?? 4096);
    setLocalSystemPrompt(p.systemPrompt ?? "");
  });

  const handleWebSearchToggle = () => {
    props.setProperties(prev => ({
      ...prev,
      webSearchEnabled: !prev.webSearchEnabled
    }));
  };

  const handleTemperatureChange = (value: number) => {
    setLocalTemp(value);
    props.setProperties(prev => ({
      ...prev,
      temperature: value
    }));
  };

  const handleMaxTokensChange = (value: number) => {
    setLocalMaxTokens(value);
    props.setProperties(prev => ({
      ...prev,
      maxTokens: value
    }));
  };

  const handleSystemPromptChange = (value: string) => {
    setLocalSystemPrompt(value);
    props.setProperties(prev => ({
      ...prev,
      systemPrompt: value
    }));
  };

  return (
    <>
      {/* Backdrop for mobile */}
      <Show when={props.isOpen()}>
        <div
          class="properties-backdrop"
          onClick={props.onClose}
          aria-hidden="true"
        />
      </Show>

      <div class={`properties-sidebar ${props.isOpen() ? "open" : ""}`}>
        {/* Header */}
        <div class="properties-header">
          <div class="properties-title-group">
            <span class="properties-icon">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 15a3 3 0 100-6 3 3 0 000 6z" />
                <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
              </svg>
            </span>
            <h2 class="properties-title">Chat Settings</h2>
          </div>
          <button
            class="properties-close-btn"
            onClick={props.onClose}
            aria-label="Close settings"
          >
            <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" fill="none" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div class="properties-content">
          {/* Web Search / Grounding Section */}
          <section class="properties-section properties-section-highlight">
            <div class="section-header">
              <div class="section-icon-badge">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <circle cx="12" cy="12" r="10" />
                  <path d="M2 12h20M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z" />
                </svg>
              </div>
              <div class="section-title-group">
                <h3 class="section-title">Web Search</h3>
                <p class="section-subtitle">Ground responses with live data</p>
              </div>
            </div>

            <div class="toggle-row">
              <div class="toggle-info">
                <span class="toggle-label">Enable Grounding</span>
                <span class="toggle-hint">Search the web for current information</span>
              </div>
              <button
                class={`toggle-switch ${props.properties().webSearchEnabled ? "active" : ""}`}
                onClick={handleWebSearchToggle}
                role="switch"
                aria-checked={props.properties().webSearchEnabled}
              >
                <span class="toggle-track">
                  <span class="toggle-thumb">
                    <Show when={props.properties().webSearchEnabled}>
                      <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" stroke-width="3">
                        <path d="M20 6L9 17l-5-5" />
                      </svg>
                    </Show>
                  </span>
                </span>
              </button>
            </div>

            <Show when={props.properties().webSearchEnabled}>
              <div class="grounding-badge">
                <span class="grounding-pulse" />
                <span class="grounding-text">Grounding active</span>
              </div>
            </Show>
          </section>

          {/* Temperature Section */}
          <section class="properties-section">
            <div class="section-header">
              <div class="section-icon-badge section-icon-temp">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M14 14.76V3.5a2.5 2.5 0 00-5 0v11.26a4.5 4.5 0 105 0z" />
                </svg>
              </div>
              <div class="section-title-group">
                <h3 class="section-title">Temperature</h3>
                <p class="section-subtitle">Controls randomness in responses</p>
              </div>
            </div>

            <div class="slider-control">
              <div class="slider-labels">
                <span class="slider-label-left">Precise</span>
                <span class="slider-value">{localTemp().toFixed(2)}</span>
                <span class="slider-label-right">Creative</span>
              </div>
              <input
                type="range"
                min="0"
                max="2"
                step="0.01"
                value={localTemp()}
                onInput={(e) => handleTemperatureChange(parseFloat(e.currentTarget.value))}
                class="properties-slider"
              />
              <div class="slider-ticks">
                <span class="tick" />
                <span class="tick" />
                <span class="tick" />
                <span class="tick" />
                <span class="tick" />
              </div>
            </div>
          </section>

          {/* Max Tokens Section */}
          <section class="properties-section">
            <div class="section-header">
              <div class="section-icon-badge section-icon-tokens">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
                  <path d="M3.27 6.96L12 12.01l8.73-5.05M12 22.08V12" />
                </svg>
              </div>
              <div class="section-title-group">
                <h3 class="section-title">Max Tokens</h3>
                <p class="section-subtitle">Maximum response length</p>
              </div>
            </div>

            <div class="token-input-group">
              <input
                type="number"
                min="256"
                max="128000"
                step="256"
                value={localMaxTokens()}
                onInput={(e) => handleMaxTokensChange(parseInt(e.currentTarget.value) || 4096)}
                class="token-input"
              />
              <div class="token-presets">
                <button
                  class={`token-preset ${localMaxTokens() === 1024 ? "active" : ""}`}
                  onClick={() => handleMaxTokensChange(1024)}
                >
                  1K
                </button>
                <button
                  class={`token-preset ${localMaxTokens() === 4096 ? "active" : ""}`}
                  onClick={() => handleMaxTokensChange(4096)}
                >
                  4K
                </button>
                <button
                  class={`token-preset ${localMaxTokens() === 16384 ? "active" : ""}`}
                  onClick={() => handleMaxTokensChange(16384)}
                >
                  16K
                </button>
                <button
                  class={`token-preset ${localMaxTokens() === 32768 ? "active" : ""}`}
                  onClick={() => handleMaxTokensChange(32768)}
                >
                  32K
                </button>
              </div>
            </div>
          </section>

          {/* System Prompt Section */}
          <section class="properties-section properties-section-expand">
            <div class="section-header">
              <div class="section-icon-badge section-icon-system">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M12 20h9M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z" />
                </svg>
              </div>
              <div class="section-title-group">
                <h3 class="section-title">System Prompt</h3>
                <p class="section-subtitle">Custom instructions for the model</p>
              </div>
            </div>

            <textarea
              class="system-prompt-input"
              placeholder="You are a helpful assistant..."
              value={localSystemPrompt()}
              onInput={(e) => handleSystemPromptChange(e.currentTarget.value)}
              rows={4}
            />
            <div class="prompt-char-count">
              {localSystemPrompt().length} characters
            </div>
          </section>
        </div>

        {/* Footer */}
        <div class="properties-footer">
          <button class="reset-btn" onClick={() => {
            props.setProperties({
              webSearchEnabled: false,
              temperature: 0.7,
              maxTokens: 4096,
              systemPrompt: ""
            });
          }}>
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M1 4v6h6M23 20v-6h-6" />
              <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 013.51 15" />
            </svg>
            Reset to defaults
          </button>
        </div>
      </div>
    </>
  );
}
