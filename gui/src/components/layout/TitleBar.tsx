import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Copy, Minus, Square, X, Github } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import DaemonConnection from "@/bindings/component/layout/DaemonConnection";
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { NotificationIndicator } from "./NotificationIndicator";

function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    const detectTauri = async () => {
      try {
        await getCurrentWindow(); // If this doesn't throw, you're in Tauri
        setIsTauri(true);
      } catch {
        setIsTauri(false);
      }
    };

    detectTauri();
  }, []);

  const openGitHub = () => {
    openUrl("https://github.com/GriffonAV/griffon");
  };

  useEffect(() => {
    if (!isTauri) return; // Skip Tauri-specific code in web

    const appWindow = getCurrentWindow();
    const minimizeButton = document.getElementById("titlebar-minimize");
    const maximizeButton = document.getElementById("titlebar-maximize");
    const closeButton = document.getElementById("titlebar-close");

    // Define event listener functions
    const handleMinimize = () => appWindow.minimize();
    const handleMaximize = () => {
      appWindow.toggleMaximize();
      setIsMaximized((prev) => !prev);
    };
    const handleClose = () => appWindow.close();

    // Add event listeners
    minimizeButton?.addEventListener("click", handleMinimize);
    maximizeButton?.addEventListener("click", handleMaximize);
    closeButton?.addEventListener("click", handleClose);

    // Cleanup event listeners on component unmount
    return () => {
      minimizeButton?.removeEventListener("click", handleMinimize);
      maximizeButton?.removeEventListener("click", handleMaximize);
      closeButton?.removeEventListener("click", handleClose);
    };
  }, [isTauri]);

  return (
    <div className="m-2 pt-2 pl-6 pr-2 flex items-center  rounded-b-none" data-tauri-drag-region>
      <div className="flex items-center gap-3 min-w-0">
        <img
          src="/assets/logo.png"
          alt="Griffon Logo"
          className="h-9 w-9 object-contain"
          style={{ imageRendering: "pixelated" }}
        />

        <div className="flex flex-col leading-none">
          <span className="text-base font-semibold tracking-wide text-foreground">
            Griffon
          </span>
          <span className="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
            Desktop app
          </span>
        </div>
      </div>
      <div className="flex-1"></div>
      <NotificationIndicator />
      <DaemonConnection />
      <Tooltip>
        <TooltipTrigger asChild>
          <a target="_blank" rel="noopener noreferrer" onClick={openGitHub}>
            <Button className="cursor-pointer text-muted-foreground" variant={"ghost"}>
              <Github />7
            </Button>
          </a>
        </TooltipTrigger>
        <TooltipContent>
          <p>Follow on GitHub</p>
        </TooltipContent>
      </Tooltip>

      {
        isTauri && (
          <>
            <div className="border-r rounded-none w-0 h-6 mx-2"></div>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 rounded-md text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              id="titlebar-minimize"
              title="minimize"
            >
              <Minus className="size-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 rounded-md text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              id="titlebar-maximize"
              title="maximize"
            >
              {isMaximized ? <Copy className="size-4" /> : <Square className="size-4" />}
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 rounded-md text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              id="titlebar-close"
              title="close"
            >
              <X className="size-4" />
            </Button>
          </>
        )
      }
    </div >
  );
}

export { TitleBar };
