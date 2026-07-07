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
import GriffonFileSelectButton from "@/components/griffon/GriffonFileSelectButton";

import CleanerTable from "@/components/griffon/CleanerTable";
import CleanerCandidateList from "@/components/griffon/CleanerCandidateList";
import CleanerDeleteResult from "@/components/griffon/CleanerDeleteResult.tsx";
import CleanerRunOverview from "@/components/griffon/CleanerRunOverview.tsx";
import CleanerFileTypeSelector from "@/components/griffon/CleanerFileTypeSelector";
import CleanerDryRunToggle from "@/components/griffon/CleanerDryRunToggle.tsx";

import ScannerTable from "@/components/griffon/ScannerTable";
import ScannerTargetTable from "@/components/griffon/ScannerTargetTable"


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

    case "file_select_button":
      return <GriffonFileSelectButton {...commonProps} />;

    case "cleaner_table":
      return <CleanerTable {...commonProps} />;

    case "cleaner_candidate_list":
      return <CleanerCandidateList {...commonProps} />;

    case "cleaner_delete_result":
      return <CleanerDeleteResult {...commonProps} />;

    case "cleaner_run_overview":
      return <CleanerRunOverview {...commonProps} />;

    case "cleaner_file_type_selector":
      return <CleanerFileTypeSelector {...commonProps} />;

    case "cleaner_dry_run_toggle":
      return <CleanerDryRunToggle {...commonProps} />;

    case "scanner_table":
      return <ScannerTable {...commonProps} />;

    case "scanner_target_table":
      return <ScannerTargetTable {...commonProps} />;

    default:
      return null;
  }
}