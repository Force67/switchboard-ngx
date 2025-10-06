import { Component, createSignal, createMemo } from "solid-js";
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

  const filteredModels = createMemo(() => {
    return props.models.filter(model =>
      model.name.toLowerCase().includes(query().toLowerCase())
    );
  });

  return (
    <div class="model-panel" onClick={(e) => e.stopPropagation()}>
      <ModelSearch query={query()} onInput={setQuery} />
      <div class="divider"></div>
      <ModelList
        models={filteredModels()}
        selectedId={props.selectedId}
        onSelect={props.onSelect}
        expanded={expanded()}
      />
      <div class="divider"></div>
      <FooterBar expanded={expanded()} onToggle={() => setExpanded(!expanded())} />
    </div>
  );
};

export default ModelPickerPanel;