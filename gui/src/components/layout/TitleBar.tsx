import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Copy, Minus, Square, X, Github, Bell } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import NotificationPanel from "./NotificationPanel";

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
    openUrl("https://github.com/GriffonAV/GriffonAV");
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
    <div className="px-4 py-3 flex items-center  rounded-b-none" data-tauri-drag-region>
      <img
        src="/assets/logo_test/G-white.png"
        alt="Griffon Logo"
        style={{
          imageRendering: "pixelated",
        }}
        className="w-9 h-auto"
      />
      <div className="pl-2 font-bold">Griffon</div>
      <div className="flex-1"></div>
      <NotificationPanel />
      <Button className="cursor-pointer text-muted-foreground" variant={"ghost"}>
        <Bell />
      </Button>
      <a target="_blank" rel="noopener noreferrer" onClick={openGitHub}>
        <Button className="cursor-pointer text-muted-foreground" variant={"ghost"}>
          <Github />7
        </Button>
      </a>
      {isTauri && (
        <>
          <Button
            className="cursor-pointer"
            variant={"ghost"}
            id="titlebar-minimize"
            title="minimize"
          >
            <Minus />
          </Button>
          <Button
            className="cursor-pointer"
            variant={"ghost"}
            id="titlebar-maximize"
            title="maximize"
          >
            {isMaximized ? <Copy /> : <Square />}
          </Button>
          <Button className="cursor-pointer" variant={"ghost"} id="titlebar-close" title="close">
            <X />
          </Button>
        </>
      )}
    </div>
  );
}

export { TitleBar };
