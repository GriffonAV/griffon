import type {
  Align,
  Gap,
  Justify,
  TextAlign,
  TextVariant,
  Tone,
} from "@/components/types";

export function gapClass(gap?: Gap) {
  switch (gap) {
    case "none":
      return "gap-0";
    case "xs":
      return "gap-1";
    case "sm":
      return "gap-2";
    case "md":
      return "gap-4";
    case "lg":
      return "gap-6";
    case "xl":
      return "gap-8";
    default:
      return "gap-4";
  }
}

export function alignItemsClass(align?: Align) {
  switch (align) {
    case "start":
      return "items-start";
    case "center":
      return "items-center";
    case "end":
      return "items-end";
    case "stretch":
      return "items-stretch";
    default:
      return "items-stretch";
  }
}

export function justifyClass(justify?: Justify) {
  switch (justify) {
    case "start":
      return "justify-start";
    case "center":
      return "justify-center";
    case "end":
      return "justify-end";
    case "space-between":
      return "justify-between";
    case "space-around":
      return "justify-around";
    default:
      return "justify-start";
  }
}

export function textVariantClass(variant?: TextVariant) {
  switch (variant) {
    case "title":
      return "text-3xl font-bold tracking-tight";
    case "subtitle":
      return "text-xl font-semibold tracking-tight";
    case "caption":
      return "text-sm text-muted-foreground";
    case "status":
      return "text-sm font-medium";
    case "body":
    default:
      return "text-base";
  }
}

export function textAlignClass(align?: TextAlign) {
  switch (align) {
    case "center":
      return "text-center";
    case "right":
      return "text-right";
    case "left":
    default:
      return "text-left";
  }
}

export function toneTextClass(tone?: Tone) {
  switch (tone) {
    case "info":
      return "text-blue-600 dark:text-blue-400";
    case "success":
      return "text-green-600 dark:text-green-400";
    case "warning":
      return "text-amber-600 dark:text-amber-400";
    case "danger":
      return "text-red-600 dark:text-red-400";
    case "neutral":
    default:
      return "text-foreground";
  }
}