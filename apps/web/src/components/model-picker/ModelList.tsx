import { Component } from "solid-js";
import { ModelMeta } from "./models";
import ModelItem from "./ModelItem";

interface Props {
  models: ModelMeta[];
  selectedId?: string;
  onSelect: (id: string) => void;
  expanded: boolean;
}

const ModelList: Component<Props> = (props) => {
  const displayedModels = () => props.expanded ? props.models : props.models.slice(0, 8);

  return (
    <div class="list">
      {displayedModels().map(model => (
        <ModelItem
          model={model}
          selectedId={props.selectedId}
          onSelect={props.onSelect}
        />
      ))}
    </div>
  );
};

export default ModelList;