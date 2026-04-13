import { Badge } from "@/components/ui/badge";
import type { BadgeElement } from "@/components/types";

type Props = {
  element: BadgeElement;
};

function mapVariant(
  variant?: "default" | "secondary" | "outline" | "destructive" | "success"
): "default" | "secondary" | "outline" | "destructive" {
  switch (variant) {
    case "secondary":
      return "secondary";
    case "outline":
      return "outline";
    case "destructive":
      return "destructive";
    case "success":
      return "default";
    case "default":
    default:
      return "default";
  }
}

export default function GriffonBadge({ element }: Props) {
  const isSuccess = element.variant === "success";

  return (
    <Badge
      id={element.id}
      variant={mapVariant(element.variant)}
      className={isSuccess ? "bg-green-600 hover:bg-green-600 text-white" : ""}
    >
      {element.name}
    </Badge>
  );
}