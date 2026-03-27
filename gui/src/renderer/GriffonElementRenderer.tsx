import type { GriffonElement, GriffonActionHandler } from "@/components/types";

import GriffonText from "@/components/griffon/GriffonText";
import GriffonButton from "@/components/griffon/GriffonButton";
import GriffonInput from "@/components/griffon/GriffonInput";
import GriffonSelect from "@/components/griffon/GriffonSelect";
import GriffonSwitch from "@/components/griffon/GriffonSwitch";
import GriffonCheckbox from "@/components/griffon/GriffonCheckbox";
import GriffonTabs from "@/components/griffon/GriffonTabs";
import GriffonBadge from "@/components/griffon/GriffonBadge";
import GriffonProgress from "@/components/griffon/GriffonProgress";
import GriffonCard from "@/components/griffon/GriffonCard";
import GriffonDivider from "@/components/griffon/GriffonDivider";
import GriffonTable from "@/components/griffon/GriffonTable";
import GriffonGroup from "@/components/griffon/GriffonGroup";
import GriffonRow from "@/components/griffon/GriffonRow";
import GriffonColumn from "@/components/griffon/GriffonColumn";

type Props = {
  element: GriffonElement;
  onAction?: GriffonActionHandler;
};

export default function GriffonElementRenderer({ element, onAction }: Props) {
  switch (element.type) {
    case "text":
      return <GriffonText element={element} />;

    case "button":
      return <GriffonButton element={element} onAction={onAction} />;

    case "input":
      return <GriffonInput element={element} onAction={onAction} />;

    case "select":
      return <GriffonSelect element={element} onAction={onAction} />;

    case "switch":
      return <GriffonSwitch element={element} onAction={onAction} />;

    case "checkbox":
      return <GriffonCheckbox element={element} onAction={onAction} />;

    case "tabs":
      return <GriffonTabs element={element} onAction={onAction} />;

    case "badge":
      return <GriffonBadge element={element} />;

    case "progress":
      return <GriffonProgress element={element} />;

    case "card":
      return <GriffonCard element={element} />;

    case "divider":
      return <GriffonDivider element={element} />;

    case "table":
      return <GriffonTable element={element} />;

    case "group":
      return <GriffonGroup element={element} onAction={onAction} />;

    case "row":
      return <GriffonRow element={element} onAction={onAction} />;

    case "column":
      return <GriffonColumn element={element} onAction={onAction} />;

    default:
      return (
        <div className="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600">
          Unsupported Griffon component type: <strong>{(element as GriffonElement).type}</strong>
        </div>
      );
  }
}