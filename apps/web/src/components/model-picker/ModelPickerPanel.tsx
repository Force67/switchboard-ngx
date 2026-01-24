import { Component, createSignal, createMemo, createEffect, onMount, Show, For } from "solid-js";
import { ModelMeta, Provider } from "./models";
import ModelSearch from "./ModelSearch";
import ProviderSidebar from "./ProviderSidebar";
import ModelItem from "./ModelItem";
import "./model-picker.css";

// Provider display names for common providers
const PROVIDER_NAMES: Record<string, string> = {
  'openai': 'OpenAI',
  'anthropic': 'Anthropic',
  'google': 'Google',
  'meta-llama': 'Meta',
  'mistralai': 'Mistral',
  'deepseek': 'DeepSeek',
  'qwen': 'Qwen',
  'x-ai': 'xAI',
  'cohere': 'Cohere',
  'perplexity': 'Perplexity',
  'groq': 'Groq',
  'nvidia': 'NVIDIA',
  'amazon': 'Amazon',
  'huggingface': 'HuggingFace',
  'microsoft': 'Microsoft',
  'ai21': 'AI21',
  'together': 'Together',
  'inflection': 'Inflection',
  'minimax': 'MiniMax',
  'zhipu': 'Zhipu',
  'moonshotai': 'Moonshot',
  'alibaba': 'Alibaba',
  'bytedance': 'ByteDance',
  'bytedance-seed': 'ByteDance',
  'baidu': 'Baidu',
  'tencent': 'Tencent',
  'meituan': 'Meituan',
  'xiaomi': 'Xiaomi',
  'nousresearch': 'Nous',
  'allenai': 'AllenAI',
  'eleutherai': 'EleutherAI',
  'databricks': 'Databricks',
  'ibm-granite': 'IBM',
  'writer': 'Writer',
  'liquid': 'Liquid',
  'cognitivecomputations': 'CogComp',
  'arcee-ai': 'Arcee',
  'stepfun-ai': 'StepFun',
  'openrouter': 'OpenRouter',
};

interface Props {
  models: ModelMeta[];
  selectedIds?: string[];
  onToggle: (id: string) => void;
  multiSelect?: boolean;
  autoFocusSearch?: boolean;
  onClose?: () => void;
}

const ModelPickerPanel: Component<Props> = (props) => {
  const [query, setQuery] = createSignal("");
  const [activeProvider, setActiveProvider] = createSignal("favorites");
  const [highlightedId, setHighlightedId] = createSignal<string | undefined>();
  const [panelEl, setPanelEl] = createSignal<HTMLDivElement>();
  const [searchInput, setSearchInput] = createSignal<HTMLInputElement>();
  const [favorites, setFavorites] = createSignal<string[]>([]);

  // Load favorites from localStorage
  onMount(() => {
    panelEl()?.focus();
    if (props.autoFocusSearch !== false) {
      requestAnimationFrame(() => {
        searchInput()?.focus();
      });
    }
    const stored = localStorage.getItem('switchboard.favorites');
    if (stored) {
      try {
        setFavorites(JSON.parse(stored));
      } catch (e) {
        console.error('Failed to parse favorites', e);
      }
    }
  });

  // Persist favorites to localStorage
  createEffect(() => {
    localStorage.setItem('switchboard.favorites', JSON.stringify(favorites()));
  });

  // Set initial highlight to selected model
  createEffect(() => {
    const selected = props.selectedIds?.[0];
    if (selected && !highlightedId()) {
      setHighlightedId(selected);
    }
  });

  const toggleFavorite = (id: string) => {
    setFavorites(prev => {
      if (prev.includes(id)) {
        return prev.filter(f => f !== id);
      } else {
        return [...prev, id];
      }
    });
  };

  const isFavorite = (id: string) => favorites().includes(id);

  // Dynamically build providers from loaded models
  const dynamicProviders = createMemo((): Provider[] => {
    const providerCounts = new Map<string, number>();

    // Count models per provider
    for (const model of props.models) {
      if (model.provider) {
        providerCounts.set(model.provider, (providerCounts.get(model.provider) || 0) + 1);
      }
    }

    // Sort by count (most models first)
    const sortedProviders = [...providerCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .map(([id]) => id);

    // Build provider list with favorites first
    const providers: Provider[] = [
      { id: 'favorites', name: 'Favorites', icon: 'star' },
    ];

    for (const providerId of sortedProviders) {
      const displayName = PROVIDER_NAMES[providerId] ||
        providerId.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
      providers.push({
        id: providerId,
        name: displayName,
        icon: providerId,
      });
    }

    return providers;
  });

  // Filter models by search query and active provider
  const filteredModels = createMemo(() => {
    let models = props.models;

    // Filter by search query
    if (query()) {
      const q = query().toLowerCase();
      models = models.filter(model =>
        model.name.toLowerCase().includes(q) ||
        model.description?.toLowerCase().includes(q) ||
        model.provider?.toLowerCase().includes(q)
      );
    }

    // Filter by provider
    if (activeProvider() === 'favorites') {
      models = models.filter(model => favorites().includes(model.id));
    } else if (activeProvider() !== 'all') {
      models = models.filter(model => model.provider === activeProvider());
    }

    return models;
  });

  // Update highlighted model when filter changes
  createEffect(() => {
    const models = filteredModels();
    const currentHighlight = highlightedId();

    if (models.length === 0) {
      setHighlightedId(undefined);
      return;
    }

    if (!currentHighlight || !models.some(m => m.id === currentHighlight)) {
      // Try to highlight selected model first
      const selectedFallback = props.selectedIds?.find(id =>
        models.some(m => m.id === id)
      );
      setHighlightedId(selectedFallback ?? models[0].id);
    }
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    const models = filteredModels();
    if (models.length === 0) return;

    const current = highlightedId();
    let currentIndex = models.findIndex(m => m.id === current);
    if (currentIndex === -1) currentIndex = 0;

    let newIndex = currentIndex;

    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        newIndex = Math.max(0, currentIndex - 1);
        break;
      case 'ArrowDown':
        e.preventDefault();
        newIndex = Math.min(models.length - 1, currentIndex + 1);
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        if (current) {
          handleToggle(current);
        }
        return;
      case 'Escape':
        e.preventDefault();
        props.onClose?.();
        return;
      default:
        return;
    }

    setHighlightedId(models[newIndex].id);

    // Scroll into view
    const row = document.querySelector(`[data-model-id="${models[newIndex].id}"]`);
    row?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  };

  const handleToggle = (id: string) => {
    setHighlightedId(id);
    props.onToggle(id);
  };

  const getEmptyMessage = () => {
    if (query()) {
      return "No models match your search";
    }
    if (activeProvider() === 'favorites') {
      return "No favorite models yet";
    }
    return "No models available";
  };

  return (
    <div
      ref={setPanelEl}
      class="model-picker-panel"
      onClick={(e) => e.stopPropagation()}
      onKeyDown={handleKeyDown}
      tabindex="0"
    >
      {/* Provider sidebar */}
      <ProviderSidebar
        providers={dynamicProviders()}
        activeProvider={activeProvider()}
        onSelect={setActiveProvider}
        favoriteCount={favorites().length}
      />

      {/* Main content area */}
      <div class="model-picker-main">
        {/* Search header */}
        <div class="model-picker-header">
          <ModelSearch
            query={query()}
            onInput={setQuery}
            inputRef={setSearchInput}
          />
        </div>

        {/* Model list */}
        <div class="model-picker-list" id="model-list" role="listbox">
          <Show
            when={filteredModels().length > 0}
            fallback={
              <div class="model-picker-empty">
                <div class="model-picker-empty-icon">
                  <svg viewBox="0 0 24 24">
                    <circle cx="12" cy="12" r="10" />
                    <path d="M16 16s-1.5-2-4-2-4 2-4 2M9 9h.01M15 9h.01" />
                  </svg>
                </div>
                <span class="model-picker-empty-text">{getEmptyMessage()}</span>
                <Show when={activeProvider() === 'favorites'}>
                  <span class="model-picker-empty-hint">
                    Star models to add them to favorites
                  </span>
                </Show>
              </div>
            }
          >
            <For each={filteredModels()}>
              {(model) => (
                <div data-model-id={model.id}>
                  <ModelItem
                    model={model}
                    highlighted={highlightedId() === model.id}
                    selected={props.selectedIds?.includes(model.id) ?? false}
                    multiSelect={props.multiSelect}
                    onToggle={handleToggle}
                    onToggleFavorite={toggleFavorite}
                    isFavorite={isFavorite(model.id)}
                  />
                </div>
              )}
            </For>
          </Show>
        </div>

        {/* Footer with model count */}
        <div class="model-picker-footer">
          <span class="model-picker-count">
            {filteredModels().length} model{filteredModels().length !== 1 ? 's' : ''}
          </span>
          <div class="model-picker-shortcuts">
            <kbd>↑↓</kbd> Navigate
            <kbd>Enter</kbd> Select
            <kbd>Esc</kbd> Close
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModelPickerPanel;
