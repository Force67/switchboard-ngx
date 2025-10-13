import { Component, createSignal, createMemo, createEffect, onMount } from "solid-js";
import { ModelMeta } from "./models";
import ModelSearch from "./ModelSearch";
import ModelList from "./ModelList";
import FooterBar from "./FooterBar";
import "./model-picker.css";

interface Props {
  models: ModelMeta[];
  selectedId?: string;
  onSelect: (id: string) => void;
}

const ModelPickerPanel: Component<Props> = (props) => {
  const [query, setQuery] = createSignal("");
  const [expanded, setExpanded] = createSignal(false);
  const [highlightedId, setHighlightedId] = createSignal<string>();
  const [panelEl, setPanelEl] = createSignal<HTMLDivElement>();
  const [favorites, setFavorites] = createSignal<string[]>([]);

  createEffect(() => {
    setHighlightedId(props.selectedId || "");
  });

  onMount(() => {
    panelEl()?.focus();
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

  const handleKeyDown = (e: KeyboardEvent) => {
    const models = filteredModels();
    if (models.length === 0) return;
    const currentIndex = models.findIndex(m => m.id === highlightedId());
    let newIndex = currentIndex;
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      newIndex = Math.max(0, currentIndex - 1);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      newIndex = Math.min(models.length - 1, currentIndex + 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      props.onSelect(highlightedId());
      return;
    } else {
      return;
    }
    setHighlightedId(models[newIndex].id);
  };

  return (
    <div ref={setPanelEl} class="model-panel" onClick={(e) => e.stopPropagation()} onKeyDown={handleKeyDown} tabindex="0">
      <ModelSearch query={query()} onInput={setQuery} />
      <div class="divider"></div>
      <ModelList
        models={filteredModels()}
        highlightedId={highlightedId()}
        onSelect={props.onSelect}
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