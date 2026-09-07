import { Info, SquareArrowOutUpRight, Bug } from "lucide-react";

import { Button } from "@/components/ui/button";

import { Badge } from "@/components/ui/badge"

import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { getVersion } from '@tauri-apps/api/app';

export function ContactButton() {
  const [clickCounter, setClickCounter] = useState(0);
  const [version, setVersion] = useState("");

  const handleButtonClick = () => {
    setClickCounter((prevCount) => prevCount + 1);
    if (clickCounter >= 10) {
      // not open url but  href = https://chromedino.com/
      window.location.href = "https://chromedino.com/";
    }
  };

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" size="icon" className="cursor-pointer" title="Contact & Support">
          <Info></Info>
        </Button>
      </DialogTrigger>

      <DialogContent className="w-min" aria-describedby={undefined}>
        <DialogTitle className="text-center hidden">Contact & Support</DialogTitle>

        <div className="flex flex-col flex-1 align-middle items-center">

          <img
            src="/assets/logo.png"
            alt="Griffon Logo"
            style={{
              imageRendering: "pixelated",
            }}
            className="w-9 h-auto pb-2"
          />
          <div className="pb-2 font-bold">Griffon</div>
          <Badge asChild>
            <a onClick={handleButtonClick}>
              {version}
            </a>
          </Badge>
          <div className="h-28"></div>
          <Button variant="outline" className="w-56 mb-2 flex align-middle cursor-pointer" onClick={() => openUrl("https://griffon-av.vercel.app/")}>
            <span>Website</span>
            <div className="flex-1"></div>
            <SquareArrowOutUpRight className="ml-2" />
          </Button>

          <Button variant="outline" className="w-56 mb-2 flex align-middle cursor-pointer" onClick={() => openUrl("https://github.com/GriffonAV/griffon/issues")}>
            <span>Report an issue</span>
            <div className="flex-1"></div>
            <Bug className="ml-2" />
          </Button>
          <Button variant="outline" className="w-56" onClick={() => openUrl("https://github.com/GriffonAV/griffon?tab=security-ov-file")}>Waranty</Button>


        </div>
      </DialogContent>
    </Dialog >
  )
}
