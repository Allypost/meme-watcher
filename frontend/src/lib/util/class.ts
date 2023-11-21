import { type Falsy } from "~/types/util";

export type ClassName =
  | string
  | Record<string, boolean>
  // eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
  | Falsy;

export const cn = (...classes: (ClassName | ClassName[])[]) =>
  classes
    .flat()
    .map((x) => {
      if (typeof x === "string") {
        return x;
      }

      if (x && typeof x === "object") {
        return Object.entries(x)
          .filter(([_className, shouldShow]) => shouldShow)
          .map(([className]) => className)
          .join(" ");
      }

      return x;
    })
    .filter(Boolean)
    .join(" ");
