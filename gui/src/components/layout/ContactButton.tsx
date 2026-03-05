import { MessageCircleQuestionMark } from "lucide-react";

import { Button } from "@/components/ui/button";

export function ContactButton() {
  return (
    <Button variant="outline" size="icon" className="cursor-pointer">
      <MessageCircleQuestionMark></MessageCircleQuestionMark>
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}
