import { Component, createSignal, createMemo, createEffect, onMount } from "solid-js";
import { ModelMeta } from "./models";
import ModelSearch from "./ModelSearch";
import ModelList from "./ModelList";
import FooterBar from "./FooterBar";
import "./model-picker.css";

interface Props {
  models: ModelMeta[];
  selectedIds?: string[];
  onToggle: (id: string) => void;
  multiSelect?: boolean;
  autoFocusSearch?: boolean;
}

const ModelPickerPanel: Component<Props> = (props) => {
  const [query, setQuery] = createSignal("");
  const [expanded, setExpanded] = createSignal(false);
  const [highlightedId, setHighlightedId] = createSignal<string | undefined>();
  const [panelEl, setPanelEl] = createSignal<HTMLDivElement>();
  const [searchInput, setSearchInput] = createSignal<HTMLInputElement>();
  const [favorites, setFavorites] = createSignal<string[]>([]);

  createEffect(() => {
    const selected = props.selectedIds?.[0];
    if (selected) {
      setHighlightedId(selected);
    }
  });

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

  createEffect(() => {
    localStorage.setItem('switchboard.favorites', JSON.stringify(favorites()));
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

  const filteredModels = createMemo(() => {
    let models = props.models.filter(model =>
      model.name.toLowerCase().includes(query().toLowerCase())
    );
    models.sort((a, b) => {
      const aFav = favorites().includes(a.id);
      const bFav = favorites().includes(b.id);
      if (aFav && !bFav) return -1;
      if (!aFav && bFav) return 1;
      return 0;
    });
    return models;
  });

  createEffect(() => {
    const currentHighlight = highlightedId();
    const models = filteredModels();
    if (models.length === 0) {
      setHighlightedId(undefined);
      return;
    }

    if (!currentHighlight || !models.some((model) => model.id === currentHighlight)) {
      const selectedFallback = props.selectedIds?.find((id) =>
        models.some((model) => model.id === id)
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
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      newIndex = Math.max(0, currentIndex - 1);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      newIndex = Math.min(models.length - 1, currentIndex + 1);
    } else if (e.key === ' ') {
      e.preventDefault();
      const current = highlightedId();
      if (current) {
        handleToggle(current);
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const current = highlightedId();
      if (current) {
        handleToggle(current);
      }
      return;
    } else {
      return;
    }
    setHighlightedId(models[newIndex].id);
  };

  const handleToggle = (id: string) => {
    setHighlightedId(id);
    props.onToggle(id);
  };

  return (
    <div ref={setPanelEl} class="model-panel" onClick={(e) => e.stopPropagation()} onKeyDown={handleKeyDown} tabindex="0">
      <ModelSearch query={query()} onInput={setQuery} inputRef={setSearchInput} />
      <div class="divider"></div>
      <ModelList
        models={filteredModels()}
        highlightedId={highlightedId()}
        selectedIds={props.selectedIds ?? []}
        multiSelect={props.multiSelect}
        onToggle={handleToggle}
        onToggleFavorite={toggleFavorite}
        isFavorite={isFavorite}
        expanded={expanded()}
      />
      <div class="divider"></div>
      <FooterBar expanded={expanded()} onToggle={() => setExpanded(!expanded())} />
    </div>
  );
};

export default ModelPickerPanel;
