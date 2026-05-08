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
import GriffonFileSelect from "@/components/griffon/GriffonFileSelect";
import CleanerTable from "@/components/griffon/CleanerTable";

interface GriffonElementRendererProps {
  element: any;
  store?: Record<string, any>;
  onAction?: (action: string, event?: any) => void;
}

export default function GriffonElementRenderer({
  element,
  store = {},
  onAction,
}: GriffonElementRendererProps) {
  const commonProps = {
    element,
    store,
    onAction,
  };

  switch (element.type) {
    case "text":
      return <GriffonText {...commonProps} />;

    case "button":
      return <GriffonButton {...commonProps} />;

    case "group":
      return <GriffonGroup {...commonProps} />;

    case "card":
      return <GriffonCard {...commonProps} />;

    case "column":
      return <GriffonColumn {...commonProps} />;

    case "row":
      return <GriffonRow {...commonProps} />;

    case "divider":
      return <GriffonDivider {...commonProps} />;

    case "badge":
      return <GriffonBadge {...commonProps} />;

    case "checkbox":
      return <GriffonCheckbox {...commonProps} />;

    case "input":
      return <GriffonInput {...commonProps} />;

    case "progress":
      return <GriffonProgress {...commonProps} />;

    case "select":
      return <GriffonSelect {...commonProps} />;

    case "switch":
      return <GriffonSwitch {...commonProps} />;

    case "table":
      return <GriffonTable {...commonProps} />;

    case "tabs":
      return <GriffonTabs {...commonProps} />;
    
    case "file_select":
      return <GriffonFileSelect {...commonProps} />;

    case "cleaner_table":
        return <CleanerTable {...commonProps} />;

    default:
      return null;
  }
}