import type { FocusableElement } from "./tabbable.js";
import { NOOP } from "../../utils/noop.js";
interface Options {
  preventScroll?: boolean | undefined;
  cancelPrevious?: boolean | undefined;
  sync?: boolean | undefined;
}
export declare function enqueueFocus(el: FocusableElement | null, options?: Options): typeof NOOP;
export {};