import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { CardElement } from "../types";

type Props = {
  element: CardElement;
};

export default function GriffonCard({ element }: Props) {
  return (
    <Card id={element.id}>
      <CardHeader>
        <CardTitle>{element.name}</CardTitle>
        {element.description ? (
          <CardDescription>{element.description}</CardDescription>
        ) : null}
      </CardHeader>
      <CardContent />
    </Card>
  );
}