import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

type Plugin = {
  uuid: string;
  display_name: string;
  file_name: string;
  version: string;
  author: string;
  description?: string;
};

type DeletePluginTableProps = {
  plugins: Plugin[];
  pluginBeingDeleted: string | null;
  onDelete: (pluginUuid: string, pluginDisplayName: string) => void | Promise<void>;
};

export function DeletePluginTable({
  plugins,
  pluginBeingDeleted,
  onDelete,
}: DeletePluginTableProps) {
  if (plugins.length === 0) {
    return <p className="mt-4 text-sm text-muted-foreground">No installed plugin found.</p>;
  }

  return (
    <div className="mt-4 w-full overflow-hidden rounded-md border border-border">
      <div className="max-h-64 overflow-y-auto">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border bg-muted/50 text-xs text-muted-foreground sticky top-0 backdrop-blur z-10">
            <tr>
              <th className="px-4 py-3 font-medium">Plugin</th>
              <th className="px-4 py-3 font-medium">File</th>
              <th className="px-4 py-3 font-medium text-right">Action</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {plugins.map((plugin) => {
              const isDeleting = pluginBeingDeleted === plugin.uuid;

              return (
                <tr key={plugin.uuid} className="bg-card hover:bg-muted/20 transition-colors">
                  <td className="px-4 py-3 font-semibold">{plugin.display_name}</td>

                  <td className="px-4 py-3 text-muted-foreground text-xs">{plugin.file_name}</td>

                  <td className="px-4 py-3 text-right">
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button
                          variant="destructive"
                          size="sm"
                          disabled={isDeleting}
                          className="cursor-pointer min-w-24"
                        >
                          {isDeleting ? "Deleting..." : "Delete"}
                        </Button>
                      </AlertDialogTrigger>

                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>Delete {plugin.display_name}?</AlertDialogTitle>

                          <AlertDialogDescription asChild>
                            <div className="flex flex-col gap-3 mt-2 text-sm text-muted-foreground">
                              <p>
                                Are you sure you want to delete this plugin? This action cannot be
                                undone and will permanently remove the plugin files.
                              </p>

                              <div className="bg-muted p-3 rounded-md flex flex-col gap-1 text-left text-xs">
                                <p>
                                  <strong className="text-foreground font-medium">UUID:</strong>{" "}
                                  {plugin.uuid}
                                </p>

                                <p>
                                  <strong className="text-foreground font-medium">
                                    Description:
                                  </strong>{" "}
                                  {plugin.description || "None"}
                                </p>

                                <p>
                                  <strong className="text-foreground font-medium">File:</strong>{" "}
                                  {plugin.file_name}
                                </p>

                                <p>
                                  <strong className="text-foreground font-medium">Version:</strong>{" "}
                                  {plugin.version}
                                </p>

                                <p>
                                  <strong className="text-foreground font-medium">Author:</strong>{" "}
                                  {plugin.author}
                                </p>
                              </div>
                            </div>
                          </AlertDialogDescription>
                        </AlertDialogHeader>

                        <AlertDialogFooter>
                          <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>

                          <AlertDialogAction
                            disabled={isDeleting}
                            onClick={() => void onDelete(plugin.uuid, plugin.display_name)}
                            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                          >
                            {isDeleting ? "Deleting..." : "Delete"}
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}