// The raw-level wire types live in the API layer (`lib/api/level`), since that's
// what `GET /level/{stageId}` returns; re-export them here so the renderer keeps
// importing its data model from `./types`.
export type { ICheckpoint, IEnemyDbRef, IFragment, ILevel, IMapData, IPosition, IRawTile, IRouteDef, ITokenInst, IWave, IWaveAction } from "#/lib/api/level";

// Renderer-only models (not part of the wire shape): player/token deployments
// painted onto the board.
export interface IMapOperator {
    is_token?: boolean;
    icon?: string;
    char_key?: string;
    direction?: number;
}

export interface IDeploymentEntry {
    coord?: string;
    row: number;
    col: number;
    name?: string;
    icon?: string;
}

export interface IDeployment {
    operators: IDeploymentEntry[];
}
