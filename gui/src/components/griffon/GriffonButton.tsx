import { Button } from "@/components/ui/button";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from "../ui/alert-dialog";

interface GriffonButtonProps {
    element: any;
    confirmation?: "Are you sure you want to perform this action?";
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
        <div>
            {
                element.confirmation ? (
                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <Button variant={variant} size={size}>
                                {element.name}
                            </Button>
                        </AlertDialogTrigger>

                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>Are you sure?</AlertDialogTitle>
                                {/* Changed to asChild so we can render a custom div structure inside the description without HTML validation errors */}
                                <AlertDialogDescription asChild>
                                    <div className="flex flex-col gap-3 mt-2 text-sm text-muted-foreground">
                                        <p>
                                            {element.confirmation}
                                        </p>
                                        <p className="text-xs text-muted-foreground">
                                            This action cannot be undone.
                                        </p>
                                    </div>
                                </AlertDialogDescription>
                            </AlertDialogHeader>

                            <AlertDialogFooter>
                                <AlertDialogCancel>Cancel</AlertDialogCancel>
                                <AlertDialogAction
                                    onClick={handleClick}
                                    className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                >
                                    Confirm
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                ) : (

                    <Button variant={variant} size={size} onClick={handleClick}>
                        {element.name}
                    </Button>
                )

            }
        </div >
    );
}