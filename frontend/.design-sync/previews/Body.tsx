import { Body } from "frontend";
import type React from "react";

const AMIYA = {
    bodyUrl: "/textures/avg/characters/avg_1037_amiya3_1/avg_1037_amiya3_1$2.png",
    faceUrl: "/textures/avg/characters/avg_1037_amiya3_1/1$2.png",
    facePos: { x: 570, y: 233, w: 117, h: 95 },
    bodySize: { w: 1280, h: 1280 },
    plate: { x: 0, y: 155, w: 1070, h: 1070 },
};
const KALTSIT = {
    bodyUrl: "/textures/avg/characters/avg_003_kalts_1/avg_003_kalts_1$1.png",
    faceUrl: "/textures/avg/characters/avg_003_kalts_1/1$1.png",
    facePos: { x: 551, y: 62, w: 136, h: 181 },
    bodySize: { w: 1280, h: 1280 },
    plate: { x: 0, y: 180, w: 890, h: 890 },
};
const ROSMON = {
    bodyUrl: "/textures/avg/characters/avg_391_rosmon_1/avg_391_rosmon_1$2.png",
    faceUrl: "/textures/avg/characters/avg_391_rosmon_1/1$2.png",
    facePos: { x: 496, y: 250, w: 144, h: 149 },
    bodySize: { w: 1024, h: 1024 },
    plate: { x: -100, y: 270, w: 1220, h: 1220 },
};

// A character's drawn body: the body texture filling its parent, with the
// expression face composited on top at `facePos` (fractions of the body
// texture). It is `absolute inset-0`, so the parent is the plate; Sprite and
// the interlude window give it one. Here the plate is a square frame.

function Plate({ children }: { children: React.ReactNode }) {
    return (
        <div className="flex w-full justify-center rounded-lg p-4" style={{ background: "linear-gradient(to bottom, #262626, #0a0a0a)" }}>
            <div className="relative" style={{ width: 384, height: 384 }}>{children}</div>
        </div>
    );
}

// Amiya (Episode 15 hub): body and face composited.
export const Amiya = () => (
    <Plate>
        <Body state={{ sprite: AMIYA, name: "amiya3" } as never} sec={0} mode="hold" />
    </Plate>
);

// Kal'tsit: a taller face rect, at the top of the texture.
export const Kaltsit = () => (
    <Plate>
        <Body state={{ sprite: KALTSIT, name: "kalts" } as never} sec={0} mode="hold" />
    </Plate>
);

// Rosmontis on a 1024 texture.
export const Rosmontis = () => (
    <Plate>
        <Body state={{ sprite: ROSMON, name: "rosmon" } as never} sec={0} mode="hold" />
    </Plate>
);
