import { Marker } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

/** A 48px board cell on the map's dark surface — the box Tile hands Marker. */
const Cell = ({ label, children }: { label: string; children?: React.ReactNode }) => (
    <div className="flex flex-col items-center gap-2">
        <div className="relative h-12 w-12" style={{ background: "hsla(0,0%,71%,0.6)", border: "1px solid hsla(0,0%,87%,0.9)", borderRadius: 1 }}>
            <div className="absolute h-full w-full p-1">{children}</div>
        </div>
        <span className="font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.12em]">{label}</span>
    </div>
);

const Surface = ({ children }: { children?: React.ReactNode }) => (
    <div className="flex w-fit flex-wrap items-start gap-8 rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px] p-8">{children}</div>
);

/** A predefined squad: `is_token: false` draws the white ring and the yellow facing wedge. */
export const DeployedOperators = () => (
    <Surface>
        <Cell label="Mlynar">
            <Marker operator={{ is_token: false, direction: 0, char_key: "char_4064_mlynar", icon: AVATAR("char_4064_mlynar") }} />
        </Cell>
        <Cell label="Skadi">
            <Marker operator={{ is_token: false, direction: 1, char_key: "char_263_skadi", icon: AVATAR("char_263_skadi") }} />
        </Cell>
        <Cell label="Texas">
            <Marker operator={{ is_token: false, direction: 2, char_key: "char_102_texas", icon: AVATAR("char_102_texas") }} />
        </Cell>
        <Cell label="Amiya">
            <Marker operator={{ is_token: false, direction: 3, char_key: "char_002_amiya", icon: AVATAR("char_002_amiya") }} />
        </Cell>
    </Surface>
);

/** `direction` 0-3 rotates the facing wedge a quarter turn each. */
export const FacingDirections = () => (
    <Surface>
        {["Up", "Right", "Down", "Left"].map((label, dir) => (
            <Cell key={label} label={label}>
                <Marker operator={{ is_token: false, direction: dir, char_key: "char_4064_mlynar", icon: AVATAR("char_4064_mlynar") }} />
            </Cell>
        ))}
    </Surface>
);

/** Level-placed tokens (`is_token: true`, the default) drop the ring and the wedge. */
export const TokenVersusOperator = () => (
    <Surface>
        <Cell label="Token">
            <Marker operator={{ is_token: true, char_key: "char_180_amgoat", icon: AVATAR("char_180_amgoat") }} />
        </Cell>
        <Cell label="Operator">
            <Marker operator={{ is_token: false, direction: 0, char_key: "char_180_amgoat", icon: AVATAR("char_180_amgoat") }} />
        </Cell>
    </Surface>
);
