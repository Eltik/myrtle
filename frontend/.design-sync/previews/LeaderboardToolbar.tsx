import { LeaderboardToolbar } from "frontend";

const noop = () => {};

export const Default = () => <LeaderboardToolbar interval="1 day" movementOnly={false} onInterval={noop} onMovementOnly={noop} onQuery={noop} onScope={noop} onServer={noop} query="" scope="global" server="All" />;

export const MovementOnly = () => <LeaderboardToolbar interval="7 days" movementOnly={true} onInterval={noop} onMovementOnly={noop} onQuery={noop} onScope={noop} onServer={noop} query="" scope="global" server="JP" />;

export const Searching = () => <LeaderboardToolbar interval="30 days" movementOnly={false} onInterval={noop} onMovementOnly={noop} onQuery={noop} onScope={noop} onServer={noop} query="Ceylonade" scope="global" server="CN" />;
