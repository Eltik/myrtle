import type { IMapOperator } from "../impl/types";
import { cx } from "../impl/util/cx";

const ICON = "/marker.png";

const FLOAT_BORDER = ["border-t-[hsla(0,0%,100%,0.9)]", "border-r-[hsla(0,0%,100%,0.9)]", "border-b-[hsla(0,0%,100%,0.9)]", "border-l-[hsla(0,0%,100%,0.9)]"];
const DIR_ROTATE = ["rotate-0", "rotate-90", "rotate-180", "rotate-[270deg]"];

export function Marker({ operator }: { operator: IMapOperator }) {
    const isToken = operator.is_token !== false;
    const dir = operator.direction ?? 0;

    return (
        <div className="transform-3d relative flex h-full w-full flex-col items-center justify-center">
            <div className={cx("translate-z-px absolute overflow-hidden rounded-full border-[3px] border-transparent [transition:border_0.8s_ease]", !isToken && FLOAT_BORDER[dir])}>
                <img src={operator.icon || ICON} alt={operator.char_key || ""} style={{ width: "40px", height: "40px" }} />
            </div>
            {!isToken && (
                <span className={cx("absolute h-full w-full", DIR_ROTATE[dir])}>
                    <span className="transform-[scale(0.4,0.8)] absolute -mt-8.75 box-border h-10 w-10 rounded-xl border-20 border-transparent border-b-[#f9bc06]" />
                </span>
            )}
            <span className="absolute h-full w-full" />
        </div>
    );
}
