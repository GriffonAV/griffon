export type Gap = "none" | "xs" | "sm" | "md" | "lg" | "xl";
export type Align = "start" | "center" | "end" | "stretch";
export type Justify =
  | "start"
  | "center"
  | "end"
  | "space-between"
  | "space-around";

export type TextVariant = "body" | "title" | "subtitle" | "caption" | "status";
export type Tone = "neutral" | "info" | "success" | "warning" | "danger";
export type TextAlign = "left" | "center" | "right";

export type BaseElement = {
  type: string;
  id?: string;
  name?: string;
  bind?: string;
  visible_if?: string;
  disabled_if?: string;
};

export type TextElement = BaseElement & {
  type: "text";
  name: string;
  variant?: TextVariant;
  tone?: Tone;
  align?: TextAlign;
};

export type ButtonElement = BaseElement & {
  type: "button";
  name: string;
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  full_width?: boolean;
  icon?: string;
  action?: string;
};

export type DividerElement = BaseElement & {
  type: "divider";
  orientation?: "horizontal" | "vertical";
  label?: string;
  tone?: Tone;
};

export type CardElement = BaseElement & {
  type: "card";
  name: string;
  description?: string;
  tone?: Tone;
  children?: GriffonElement[];
};

export type GroupElement = BaseElement & {
  type: "group";
  name?: string;
  title?: string;
  description?: string;
  gap?: Gap;
  children?: GriffonElement[];
};

export type RowElement = BaseElement & {
  type: "row";
  name?: string;
  gap?: Gap;
  align?: Align;
  justify?: Justify;
  wrap?: boolean;
  children?: GriffonElement[];
};

export type ColumnElement = BaseElement & {
  type: "column";
  name?: string;
  gap?: Gap;
  align?: Align;
  justify?: Justify;
  children?: GriffonElement[];
};

export type InputElement = BaseElement & {
  type: "input";
  label?: string;
  placeholder?: string;
  description?: string;
  value?: string;
  input_type?: "text" | "email" | "password" | "number" | "search" | "url";
  disabled?: boolean;
  action?: string;
};

export type SelectOption = {
  label: string;
  value: string;
};

export type SelectElement = BaseElement & {
  type: "select";
  label?: string;
  placeholder?: string;
  description?: string;
  value?: string;
  disabled?: boolean;
  options?: SelectOption[];
  action?: string;
};

export type FileSelectElement = BaseElement & {
  type: "file_select";
  label?: string;
  placeholder?: string;
  description?: string;
  value?: string;
  button_label?: string;
  accept?: string;
  disabled?: boolean;
  action?: string;
};

export type SwitchElement = BaseElement & {
  type: "switch";
  label: string;
  description?: string;
  checked?: boolean;
  disabled?: boolean;
  action?: string;
};

export type CheckboxElement = BaseElement & {
  type: "checkbox";
  label: string;
  description?: string;
  checked?: boolean;
  disabled?: boolean;
  action?: string;
};

export type TabsItem = {
  label: string;
  value: string;
  children?: GriffonElement[];
};

export type TabsElement = BaseElement & {
  type: "tabs";
  value?: string;
  tabs?: TabsItem[];
  action?: string;
};

export type BadgeElement = BaseElement & {
  type: "badge";
  name: string;
  variant?: "default" | "secondary" | "outline" | "destructive" | "success";
};

export type ProgressElement = BaseElement & {
  type: "progress";
  label?: string;
  value: number;
  show_value?: boolean;
};

export type TableColumn = {
  key: string;
  label: string;
};

export type TableRowData = {
  id?: string;
  [key: string]: string | number | boolean | null | undefined;
};

export type TableElement = BaseElement & {
  type: "table";
  columns?: TableColumn[];
  rows?: TableRowData[];
};

export type GriffonInteractionStep =
  | {
      type: "set";
      key: string;
      value?: string | number | boolean;
      from?: string;
    }
  | {
      type: "toggle";
      key: string;
    }
  | {
      type: "increment";
      key: string;
      by?: number;
    }
  | {
      type: "decrement";
      key: string;
      by?: number;
    };

export type GriffonInteraction = {
  id: string;
  on: string;
  steps: GriffonInteractionStep[];
};

export type GriffonSection = {
  id: string;
  tab: string;
  contents: GriffonElement[];
};

export type GriffonPlugin = {
  name: string;
  id: string;
  version: string;
  author: string;
  description?: string;
  tabs: string[];
};

export type GriffonManifest = {
  plugin: GriffonPlugin;
  ui: {
    sections: GriffonSection[];
  };
  store?: Record<string, string | number | boolean | null>;
  interactions?: GriffonInteraction[];
};

export type CleanerCandidateListValue = {
  paths: string[];
  dry_run?: boolean;
  file_types?: string[];
};

export type CleanerCandidateListElement = {
  type: "cleaner_candidate_list";
  id: string;
  title?: string;
  from?: string;
  selectedFrom?: string;
  action?: string;
  value?: CleanerCandidateListValue;
  optionsFrom?: string;
  deleteAction?: string;
};

export type GriffonActionHandler = (
  action: string,
  element: GriffonElement
) => void;

export type CleanerFileTypeSelectorElement = {
  type: "cleaner_file_type_selector";
  id: string;
  title?: string;
  description?: string;
  from?: string;
  selectedFrom?: string;
  action?: string;
  value?: {
    file_types: string[];
  };
};

export type CleanerDryRunToggleElement = {
  type: "cleaner_dry_run_toggle";
  id: string;
  title?: string;
  description?: string;
  from?: string;
  action?: string;
  value?: {
    dry_run: boolean;
  };
};

export type GriffonElement =
  | TextElement
  | ButtonElement
  | InputElement
  | SelectElement
  | FileSelectElement
  | SwitchElement
  | CheckboxElement
  | TabsElement
  | BadgeElement
  | ProgressElement
  | CardElement
  | DividerElement
  | TableElement
  | GroupElement
  | RowElement
  | ColumnElement
  | CleanerCandidateListElement
  | CleanerFileTypeSelectorElement
  | CleanerDryRunToggleElement;