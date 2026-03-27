import { Button } from "@/components/ui/button";
import type { ButtonElement, GriffonActionHandler } from "../types";

type Props = {
  element: ButtonElement;
  onAction?: GriffonActionHandler;
};

function mapVariant(
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success"
): "default" | "secondary" | "ghost" | "destructive" {
  switch (variant) {
    case "secondary":
      return "secondary";
    case "ghost":
      return "ghost";
    case "danger":
      return "destructive";
    case "success":
      // shadcn has no built-in "success", so fallback to default
      return "default";
    case "primary":
    default:
      return "default";
  }
}

function mapSize(size?: "sm" | "md" | "lg"): "sm" | "default" | "lg" {
  switch (size) {
    case "sm":
      return "sm";
    case "lg":
      return "lg";
    case "md":
    default:
      return "default";
  }
}

export default function GriffonButton({ element, onAction }: Props) {
  const isSuccess = element.variant === "success";

  return (
    <Button
      id={element.id}
      variant={mapVariant(element.variant)}
      size={mapSize(element.size)}
      disabled={element.disabled}
      onClick={() => {
        if (element.action && onAction) {
          onAction(element.action, element);
        }
      }}
      className={[
        element.full_width ? "w-full" : "",
        isSuccess ? "bg-green-600 hover:bg-green-700 text-white" : "",
      ].join(" ")}
    >
      {element.name}
    </Button>
  );
}