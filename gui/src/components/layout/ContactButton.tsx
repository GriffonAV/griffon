import { Info, SquareArrowOutUpRight, Bug } from "lucide-react";

import { Button } from "@/components/ui/button";

import { Badge } from "@/components/ui/badge"

import {
  Dialog,
  DialogContent,

  DialogTrigger,
} from "@/components/ui/dialog"
import { openUrl } from "@tauri-apps/plugin-opener";

export function ContactButton() {
  return (
    <Dialog>
      <DialogTrigger><Button variant="outline" size="icon" className="cursor-pointer">
        <Info></Info>
        <span className="sr-only">Toggle theme</span>
      </Button></DialogTrigger>

      <DialogContent className="w-min">

        <div className="flex flex-col flex-1 align-middle items-center">

          <img
            src="/assets/logo_test/G-white.png"
            alt="Griffon Logo"
            style={{
              imageRendering: "pixelated",
            }}
            className="w-9 h-auto pb-2"
          />
          <div className="pb-2 font-bold">Griffon</div>
          <Badge asChild>
            <a href="#link">
              0.0.1
            </a>
          </Badge>
          <div className="h-28"></div>
          <Button variant="outline" className="w-56 mb-2 flex align-middle cursor-pointer" onClick={() => openUrl("https://github.com/GriffonAV/GriffonAV")}>
            <span>Website</span>
            <div className="flex-1"></div>
            <SquareArrowOutUpRight className="ml-2" />
          </Button>

          <Button variant="outline" className="w-56 mb-2 flex align-middle cursor-pointer" onClick={() => openUrl("https://github.com/GriffonAV/GriffonAV/issues")}>
            <span>Report an issue</span>
            <div className="flex-1"></div>
            <Bug className="ml-2" />
          </Button>
          <Button variant="outline" className="w-56">Waranty</Button>


        </div>
      </DialogContent>
    </Dialog>
  )
}
