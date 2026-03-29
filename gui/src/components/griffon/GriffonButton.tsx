import { Button } from "@/components/ui/button";

interface GriffonButtonProps {
    element: any;
    onAction?: (action: string, event?: any) => void;
}

export default function GriffonButton({
    element,
    onAction,
}: GriffonButtonProps) {
    const variant =
        element.variant === "primary"
            ? "default"
            : element.variant === "secondary"
                ? "secondary"
                : element.variant === "outline"
                    ? "outline"
                    : "default";

    const size =
        element.size === "sm"
            ? "sm"
            : element.size === "lg"
                ? "lg"
                : "default";

    function handleClick() {
        if (element.action) {
            onAction?.(element.action, {
                source: element.id,
                type: "click",
            });
        }
    }

    return (
        <Button variant={variant} size={size} onClick={handleClick}>
            {element.name}
        </Button>
    );
}